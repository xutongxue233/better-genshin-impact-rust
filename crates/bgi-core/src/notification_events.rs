use super::notification_model::NotificationEventDescriptor;

pub fn notification_events() -> Vec<NotificationEventDescriptor> {
    NOTIFICATION_EVENTS.to_vec()
}

pub fn parse_notification_event_codes(subscribe_event_str: Option<&str>) -> Vec<String> {
    let Some(subscribe_event_str) = subscribe_event_str else {
        return Vec::new();
    };
    let mut event_codes = Vec::new();
    for event_code in subscribe_event_str.split(',') {
        let trimmed = event_code.trim();
        if trimmed.is_empty() {
            continue;
        }
        if event_codes
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(trimmed))
        {
            continue;
        }
        event_codes.push(trimmed.to_string());
    }
    event_codes
}

pub fn normalize_notification_event_codes<'a>(
    event_codes: impl IntoIterator<Item = &'a str>,
) -> String {
    let mut normalized = Vec::new();
    for event_code in event_codes {
        let trimmed = event_code.trim();
        if trimmed.is_empty() {
            continue;
        }
        if normalized
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(trimmed))
        {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    normalized.join(",")
}

pub fn should_send_notification(
    subscribe_event_str: Option<&str>,
    event_code: Option<&str>,
) -> bool {
    let Some(subscribe_event_str) = subscribe_event_str else {
        return true;
    };
    if subscribe_event_str.trim().is_empty() {
        return true;
    }

    let Some(event_code) = event_code else {
        return false;
    };
    if event_code.trim().is_empty() {
        return false;
    }

    parse_notification_event_codes(Some(subscribe_event_str))
        .iter()
        .any(|subscribed| subscribed.eq_ignore_ascii_case(event_code))
}

const NOTIFICATION_EVENTS: &[NotificationEventDescriptor] = &[
    NotificationEventDescriptor {
        code: "notify.test",
        message: "测试通知",
    },
    NotificationEventDescriptor {
        code: "domain.reward",
        message: "自动秘境奖励",
    },
    NotificationEventDescriptor {
        code: "domain.start",
        message: "自动秘境启动",
    },
    NotificationEventDescriptor {
        code: "domain.end",
        message: "自动秘境结束",
    },
    NotificationEventDescriptor {
        code: "domain.retry",
        message: "自动秘境重试",
    },
    NotificationEventDescriptor {
        code: "task.cancel",
        message: "任务启动",
    },
    NotificationEventDescriptor {
        code: "task.error",
        message: "任务错误",
    },
    NotificationEventDescriptor {
        code: "group.start",
        message: "配置组启动",
    },
    NotificationEventDescriptor {
        code: "group.end",
        message: "配置组结束",
    },
    NotificationEventDescriptor {
        code: "dragon.start",
        message: "一条龙启动",
    },
    NotificationEventDescriptor {
        code: "dragon.end",
        message: "一条龙结束",
    },
    NotificationEventDescriptor {
        code: "tcg.start",
        message: "七圣召唤启动",
    },
    NotificationEventDescriptor {
        code: "tcg.end",
        message: "七圣召唤结束",
    },
    NotificationEventDescriptor {
        code: "album.start",
        message: "自动音游专辑启动",
    },
    NotificationEventDescriptor {
        code: "album.end",
        message: "自动音游专辑结束",
    },
    NotificationEventDescriptor {
        code: "album.error",
        message: "自动音游专辑错误",
    },
    NotificationEventDescriptor {
        code: "daily.reward",
        message: "检查每日奖励领取状态",
    },
    NotificationEventDescriptor {
        code: "js.custom",
        message: "JS自定义事件",
    },
    NotificationEventDescriptor {
        code: "js.error",
        message: "JS运行时错误",
    },
    NotificationEventDescriptor {
        code: "autoeat.start",
        message: "自动吃药启动",
    },
    NotificationEventDescriptor {
        code: "autoeat.end",
        message: "自动吃药结束",
    },
    NotificationEventDescriptor {
        code: "autoeat.info",
        message: "自动吃药信息",
    },
];
