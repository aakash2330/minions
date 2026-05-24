use crate::domain::{MessageRole, Session, SessionKind, SessionStatus, WorkspaceChatMessage};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkspaceChatTurnRole {
    Primary,
    Secondary,
}

impl WorkspaceChatTurnRole {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WorkspaceChatTurnOrigin {
    pub(crate) workspace_id: String,
    pub(crate) user_message_id: String,
    pub(crate) response_message_id: String,
    pub(crate) role: WorkspaceChatTurnRole,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkspaceChatParticipant {
    pub(crate) session_id: String,
    pub(crate) name: String,
    pub(crate) kind: SessionKind,
    pub(crate) status: SessionStatus,
    pub(crate) role: WorkspaceChatTurnRole,
}

const CODE_KEYWORDS: &[&str] = &[
    "code",
    "coding",
    "file",
    "files",
    "edit",
    "edits",
    "debug",
    "test",
    "tests",
    "refactor",
    "implement",
    "implementation",
    "update",
    "fix",
    "build",
];

const RESEARCH_KEYWORDS: &[&str] = &[
    "research",
    "docs",
    "documentation",
    "sources",
    "source",
    "materials",
    "material",
    "compare",
    "latest",
    "library",
    "api",
    "paper",
    "benchmark",
    "weather",
    "forecast",
    "lookup",
];

pub(crate) fn select_workspace_chat_participants(
    prompt: &str,
    sessions: &[Session],
) -> Vec<WorkspaceChatParticipant> {
    let routeable_sessions = sessions
        .iter()
        .filter(|session| matches!(session.kind, SessionKind::Coder | SessionKind::Researcher))
        .collect::<Vec<_>>();

    let explicit = routeable_sessions
        .iter()
        .filter_map(|session| {
            explicitly_mentions_session(prompt, session)
                .then(|| participant(session, WorkspaceChatTurnRole::Primary))
        })
        .collect::<Vec<_>>();

    if !explicit.is_empty() || contains_workspace_chat_ping(prompt) {
        return explicit;
    }

    if contains_word(prompt, "everyone")
        || contains_word(prompt, "everybody")
        || contains_word(prompt, "all")
    {
        return routeable_sessions
            .into_iter()
            .map(|session| participant(session, WorkspaceChatTurnRole::Primary))
            .collect();
    }

    let has_code_intent = CODE_KEYWORDS
        .iter()
        .any(|keyword| contains_word(prompt, keyword));
    let has_research_intent = RESEARCH_KEYWORDS
        .iter()
        .any(|keyword| contains_word(prompt, keyword));

    if has_code_intent || has_research_intent {
        let mut selected = Vec::new();

        if has_code_intent {
            selected.extend(
                routeable_sessions
                    .iter()
                    .filter(|session| session.kind == SessionKind::Coder)
                    .map(|session| participant(session, WorkspaceChatTurnRole::Primary)),
            );
        }

        if has_research_intent {
            let role = if has_code_intent {
                WorkspaceChatTurnRole::Secondary
            } else {
                WorkspaceChatTurnRole::Primary
            };
            selected.extend(
                routeable_sessions
                    .iter()
                    .filter(|session| session.kind == SessionKind::Researcher)
                    .map(|session| participant(session, role)),
            );
        }

        if !selected.is_empty() {
            return selected;
        }
    }

    routeable_sessions
        .into_iter()
        .map(|session| participant(session, WorkspaceChatTurnRole::Primary))
        .collect()
}

pub(crate) fn no_workspace_chat_participants_message(prompt: &str) -> &'static str {
    if contains_workspace_chat_ping(prompt) {
        "No matching coder or researcher sessions were found for that ping."
    } else {
        "No coder or researcher sessions are available for this workspace."
    }
}

pub(crate) fn build_workspace_chat_turn_prompt(
    prompt: &str,
    role: WorkspaceChatTurnRole,
    history: &[WorkspaceChatMessage],
) -> String {
    let history_text = if history.is_empty() {
        "No prior global chat messages.".to_owned()
    } else {
        history
            .iter()
            .map(format_history_message)
            .collect::<Vec<_>>()
            .join("\n")
    };

    let role_instruction = match role {
        WorkspaceChatTurnRole::Primary => {
            "You are the primary responder for this global workspace chat turn. If the request is actionable for your role, handle it directly subject to your normal approval and verification rules."
        }
        WorkspaceChatTurnRole::Secondary => {
            "You are a secondary proactive responder for this global workspace chat turn. Add concise supporting evidence, research material, or caveats useful to the primary responder; avoid taking over implementation unless explicitly asked."
        }
    };

    format!(
        "Global workspace chat context:\n{history_text}\n\n\
Your global chat role for this turn: {}.\n\
{role_instruction}\n\n\
Current global user prompt:\n{prompt}",
        role.as_str()
    )
}

pub(crate) fn busy_participant_message(participant_name: &str) -> String {
    format!("{participant_name} is busy and was not routed this turn.")
}

fn participant(session: &Session, role: WorkspaceChatTurnRole) -> WorkspaceChatParticipant {
    WorkspaceChatParticipant {
        session_id: session.session_id.clone(),
        name: session.name.clone(),
        kind: session.kind,
        status: session.status,
        role,
    }
}

fn explicitly_mentions_session(prompt: &str, session: &Session) -> bool {
    mention_matches_target(prompt, session.session_id.as_str())
        || mention_matches_target(prompt, session.name.as_str())
        || (session.kind == SessionKind::Coder && mention_matches_target(prompt, "coder"))
        || (session.kind == SessionKind::Researcher && mention_matches_target(prompt, "researcher"))
}

fn contains_workspace_chat_ping(prompt: &str) -> bool {
    prompt.char_indices().any(|(index, character)| {
        character == '@'
            && has_mention_prefix_boundary(prompt, index)
            && prompt[index + '@'.len_utf8()..]
                .chars()
                .next()
                .is_some_and(is_mention_character)
    })
}

fn mention_matches_target(prompt: &str, target: &str) -> bool {
    let Some(target) = clean_mention_target(target) else {
        return false;
    };
    let prompt = prompt.to_lowercase();
    let mut search_from = 0;

    while let Some(relative_index) = prompt[search_from..].find('@') {
        let mention_marker = search_from + relative_index;
        let mention_start = mention_marker + '@'.len_utf8();
        let mention = &prompt[mention_start..];

        if has_mention_prefix_boundary(prompt.as_str(), mention_marker)
            && mention.starts_with(target.as_str())
            && mention[target.len()..]
                .chars()
                .next()
                .is_none_or(is_mention_boundary)
        {
            return true;
        }

        search_from = mention_start;
    }

    false
}

fn has_mention_prefix_boundary(prompt: &str, mention_marker: usize) -> bool {
    prompt[..mention_marker]
        .chars()
        .next_back()
        .is_none_or(is_mention_boundary)
}

fn clean_mention_target(target: &str) -> Option<String> {
    let target = target.trim().to_lowercase();
    (!target.is_empty()).then_some(target)
}

fn is_mention_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '-'
}

fn is_mention_boundary(character: char) -> bool {
    !is_mention_character(character)
}

fn contains_word(text: &str, word: &str) -> bool {
    text.split(|character: char| !character.is_ascii_alphanumeric())
        .any(|token| token.eq_ignore_ascii_case(word))
}

fn format_history_message(message: &WorkspaceChatMessage) -> String {
    let speaker = match message.role {
        MessageRole::User => "User".to_owned(),
        MessageRole::Assistant => message.session_id.as_ref().map_or_else(
            || "Assistant".to_owned(),
            |session_id| session_id.to_owned(),
        ),
        MessageRole::System => "System".to_owned(),
    };

    format!("- {speaker}: {}", message.text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Direction, PointWithFacing};

    #[test]
    fn file_update_routes_to_coder_only() {
        let participants =
            select_workspace_chat_participants("I want to update some file", &sessions());

        assert_eq!(participant_ids(&participants), vec!["kevin"]);
        assert_eq!(participants[0].role, WorkspaceChatTurnRole::Primary);
    }

    #[test]
    fn research_wording_adds_researcher_as_secondary_for_code_turns() {
        let participants = select_workspace_chat_participants(
            "Implement the API update and include research docs",
            &sessions(),
        );

        assert_eq!(participant_ids(&participants), vec!["kevin", "bob"]);
        assert_eq!(participants[0].role, WorkspaceChatTurnRole::Primary);
        assert_eq!(participants[1].role, WorkspaceChatTurnRole::Secondary);
    }

    #[test]
    fn explicit_mention_overrides_auto_routing() {
        let participants =
            select_workspace_chat_participants("@bob update the file research notes", &sessions());

        assert_eq!(participant_ids(&participants), vec!["bob"]);
        assert_eq!(participants[0].role, WorkspaceChatTurnRole::Primary);
    }

    #[test]
    fn explicit_ping_can_match_session_name_instead_of_id() {
        let sessions = vec![
            session(
                "research-1",
                "Bob",
                SessionKind::Researcher,
                SessionStatus::Idle,
            ),
            session("coder-1", "Kevin", SessionKind::Coder, SessionStatus::Idle),
        ];
        let participants = select_workspace_chat_participants("@Kevin update the file", &sessions);

        assert_eq!(participant_ids(&participants), vec!["coder-1"]);
        assert_eq!(participants[0].role, WorkspaceChatTurnRole::Primary);
    }

    #[test]
    fn unknown_ping_does_not_fall_back_to_auto_routing() {
        let participants =
            select_workspace_chat_participants("@alice update the file", &sessions());

        assert!(participants.is_empty());
    }

    #[test]
    fn ping_matching_uses_boundaries() {
        let participants =
            select_workspace_chat_participants("@bobby update the research notes", &sessions());

        assert!(participants.is_empty());
    }

    #[test]
    fn email_addresses_are_not_session_pings() {
        let participants = select_workspace_chat_participants("contact dev@bob.com", &sessions());

        assert_eq!(participant_ids(&participants), vec!["bob", "kevin"]);
    }

    #[test]
    fn weather_lookup_routes_to_researcher_only() {
        let participants =
            select_workspace_chat_participants("mumbai current weather please", &sessions());

        assert_eq!(participant_ids(&participants), vec!["bob"]);
        assert_eq!(participants[0].role, WorkspaceChatTurnRole::Primary);
    }

    #[test]
    fn everyone_routes_to_all_routeable_sessions() {
        let participants = select_workspace_chat_participants("everyone weigh in", &sessions());

        assert_eq!(participant_ids(&participants), vec!["bob", "kevin"]);
    }

    #[test]
    fn busy_participant_message_names_skipped_session() {
        assert_eq!(
            busy_participant_message("Kevin"),
            "Kevin is busy and was not routed this turn."
        );
    }

    #[test]
    fn no_participants_message_distinguishes_unknown_ping() {
        assert_eq!(
            no_workspace_chat_participants_message("@alice update files"),
            "No matching coder or researcher sessions were found for that ping."
        );
        assert_eq!(
            no_workspace_chat_participants_message("update files"),
            "No coder or researcher sessions are available for this workspace."
        );
    }

    fn participant_ids(participants: &[WorkspaceChatParticipant]) -> Vec<&str> {
        participants
            .iter()
            .map(|participant| participant.session_id.as_str())
            .collect()
    }

    fn sessions() -> Vec<Session> {
        vec![
            session("bob", "Bob", SessionKind::Researcher, SessionStatus::Idle),
            session("kevin", "Kevin", SessionKind::Coder, SessionStatus::Idle),
            session(
                "reviewer",
                "Reviewer",
                SessionKind::Reviewer,
                SessionStatus::Idle,
            ),
        ]
    }

    fn session(session_id: &str, name: &str, kind: SessionKind, status: SessionStatus) -> Session {
        Session {
            session_id: session_id.to_owned(),
            workspace_id: "default".to_owned(),
            name: name.to_owned(),
            kind,
            status,
            spawn: PointWithFacing {
                x: 0,
                y: 0,
                facing: Direction::Down,
            },
            current: PointWithFacing {
                x: 0,
                y: 0,
                facing: Direction::Down,
            },
            messages: Vec::new(),
        }
    }
}
