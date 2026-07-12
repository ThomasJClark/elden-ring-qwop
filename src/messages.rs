use std::sync::LazyLock;

#[repr(u32)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types, dead_code)]
pub enum MessageCategory {
    GR_MenuText = 200,
    GR_LineHelp = 201,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct StaticUtf16String(*const u16);

unsafe impl Sync for StaticUtf16String {}
unsafe impl Send for StaticUtf16String {}

impl From<&str> for StaticUtf16String {
    fn from(s: &str) -> Self {
        Self(s.encode_utf16().collect::<Vec<_>>().leak().as_ptr())
    }
}

static MESSAGES: LazyLock<QwopMessages> = LazyLock::new(QwopMessages::new);

pub struct QwopMessages {
    toggle_controls_menu_text: StaticUtf16String,
    q_menu_text: StaticUtf16String,
    w_menu_text: StaticUtf16String,
    o_menu_text: StaticUtf16String,
    p_menu_text: StaticUtf16String,
    unused1_menu_text: StaticUtf16String,
    unused2_menu_text: StaticUtf16String,
    cheese_discovered_menu_text: StaticUtf16String,
    toggle_controls_line_help: StaticUtf16String,
    q_line_help: StaticUtf16String,
    w_line_help: StaticUtf16String,
    o_line_help: StaticUtf16String,
    p_line_help: StaticUtf16String,
    unused1_line_help: StaticUtf16String,
    unused2_line_help: StaticUtf16String,
}

impl QwopMessages {
    fn new() -> Self {
        QwopMessages {
            toggle_controls_menu_text: "Toggle normal controls\0".into(),
            q_menu_text: "Thighs (Right)\0".into(),
            w_menu_text: "Thighs (Left)\0".into(),
            o_menu_text: "Calves (Right)\0".into(),
            p_menu_text: "Calves (Left)\0".into(),
            unused1_menu_text: "-\0".into(),
            unused2_menu_text: "-\0".into(),
            cheese_discovered_menu_text: "CHEESE DISCOVERED\0".into(),
            toggle_controls_line_help: "Switch between QWOP and vanilla movement controls\0".into(),
            q_line_help: "Move left thigh backwards and right thigh forwards\0".into(),
            w_line_help: "Move left thigh forwards and right thigh backwards\0".into(),
            o_line_help: "Move left calf backwards and right calf forwards\0".into(),
            p_line_help: "Move left calf forwards and right calf backwards\0".into(),
            unused1_line_help: "\0".into(),
            unused2_line_help: "\0".into(),
        }
    }

    pub fn lookup_message(category: MessageCategory, id: i32) -> Option<StaticUtf16String> {
        match (category, id) {
            (MessageCategory::GR_MenuText, 280100) => Some(MESSAGES.toggle_controls_menu_text),
            (MessageCategory::GR_MenuText, 280101) => Some(MESSAGES.q_menu_text),
            (MessageCategory::GR_MenuText, 280102) => Some(MESSAGES.w_menu_text),
            (MessageCategory::GR_MenuText, 280103) => Some(MESSAGES.o_menu_text),
            (MessageCategory::GR_MenuText, 280104) => Some(MESSAGES.p_menu_text),
            (MessageCategory::GR_MenuText, 280105) => Some(MESSAGES.unused1_menu_text),
            (MessageCategory::GR_MenuText, 280107) => Some(MESSAGES.unused2_menu_text),
            (MessageCategory::GR_MenuText, 331338) => Some(MESSAGES.cheese_discovered_menu_text),
            (MessageCategory::GR_LineHelp, 280100) => Some(MESSAGES.toggle_controls_line_help),
            (MessageCategory::GR_LineHelp, 280101) => Some(MESSAGES.q_line_help),
            (MessageCategory::GR_LineHelp, 280102) => Some(MESSAGES.w_line_help),
            (MessageCategory::GR_LineHelp, 280103) => Some(MESSAGES.o_line_help),
            (MessageCategory::GR_LineHelp, 280104) => Some(MESSAGES.p_line_help),
            (MessageCategory::GR_LineHelp, 280105) => Some(MESSAGES.unused1_line_help),
            (MessageCategory::GR_LineHelp, 280107) => Some(MESSAGES.unused2_line_help),
            _ => None,
        }
    }
}
