use super::*;

#[test]
fn update_request_plans_preserve_legacy_urls_and_alpha_query() {
    let stable = update_request_plan(UpdateOption::default());
    assert_eq!(stable.url, NOTICE_URL);
    assert!(stable.query.is_empty());

    let alpha = update_request_plan(UpdateOption {
        trigger: UpdateTrigger::Manual,
        channel: UpdateChannel::Alpha,
    });
    assert_eq!(alpha.url, MIRROR_CHYAN_LATEST_URL);
    assert_eq!(alpha.query["user_agent"], "BetterGI");
    assert_eq!(alpha.query["os"], "win");
    assert_eq!(alpha.query["arch"], "x64");
    assert_eq!(alpha.query["channel"], "alpha");
}

#[test]
fn stable_notice_applies_gray_gate_and_empty_version_guard() {
    let notice = Notice {
        version: "1.2.3".to_string(),
        gray: 10,
    };
    assert_eq!(
        latest_version_from_notice(&notice, 9),
        Some("1.2.3".to_string())
    );
    assert_eq!(
        latest_version_from_notice(&Notice { gray: 0, ..notice }, 0),
        None
    );
    assert_eq!(latest_version_from_notice(&Notice::default(), 0), None);
}

#[test]
fn mirror_chyan_outcome_matches_legacy_error_mapping() {
    let response = MirrorChyanLatestResponse {
        code: 0,
        msg: "ok".to_string(),
        data: Some(MirrorChyanLatestData {
            arch: "x64".to_string(),
            cdk_expired_time: None,
            channel: "alpha".to_string(),
            custom_data: None,
            filesize: None,
            os: "win".to_string(),
            release_note: String::new(),
            sha256: None,
            update_type: Some("full".to_string()),
            url: None,
            version_name: "0.99.0-alpha.1".to_string(),
            version_number: 99,
        }),
    };
    assert_eq!(
        mirror_chyan_latest_outcome(Some(&response)),
        MirrorChyanLatestOutcome::Version("0.99.0-alpha.1".to_string())
    );

    let warning = MirrorChyanLatestResponse {
        code: 7002,
        msg: "bad cdk".to_string(),
        data: None,
    };
    assert_eq!(
        mirror_chyan_latest_outcome(Some(&warning)),
        MirrorChyanLatestOutcome::Warning {
            code: 7002,
            message: "Mirror酱 CDK 错误!".to_string()
        }
    );

    let severe = MirrorChyanLatestResponse {
        code: -1,
        msg: "boom".to_string(),
        data: None,
    };
    assert!(matches!(
        mirror_chyan_latest_outcome(Some(&severe)),
        MirrorChyanLatestOutcome::Severe { code: -1, .. }
    ));
}

#[test]
fn update_decision_matches_legacy_manual_auto_and_ignore_rules() {
    let auto = UpdateOption::default();
    assert_eq!(
        update_decision(auto, "1.0.0", None, None).action,
        UpdateDecisionAction::Noop
    );
    assert_eq!(
        update_decision(auto, "1.0.0", None, Some("1.0.0")).action,
        UpdateDecisionAction::Noop
    );
    assert_eq!(
        update_decision(
            UpdateOption {
                trigger: UpdateTrigger::Manual,
                channel: UpdateChannel::Stable,
            },
            "1.0.0",
            None,
            Some("1.0.0"),
        )
        .action,
        UpdateDecisionAction::ShowUpToDateMessage
    );
    assert_eq!(
        update_decision(auto, "1.0.0", Some("1.2.0"), Some("1.1.0")).action,
        UpdateDecisionAction::SuppressedByIgnoredVersion
    );
    let decision = update_decision(auto, "1.0.0", Some("1.0.5"), Some("1.1.0"));
    assert_eq!(decision.action, UpdateDecisionAction::OpenUpdateWindow);
    assert_eq!(decision.download_page_url, Some(DOWNLOAD_PAGE_URL));
    assert!(decision.release_notes_request.is_some());
}

#[test]
fn updater_launch_options_preserve_legacy_sources_and_args() {
    let stable = updater_launch_options(UpdateChannel::Stable);
    assert_eq!(
        stable
            .iter()
            .map(|option| option.source)
            .collect::<Vec<_>>(),
        vec![
            UpdaterSource::Default,
            UpdaterSource::Cnb,
            UpdaterSource::Github,
            UpdaterSource::MirrorChyan,
        ]
    );
    assert_eq!(stable[0].args, ["-I"]);
    assert_eq!(stable[1].args, ["-I", "--source", "cnb"]);
    assert_eq!(stable[2].args, ["-I", "--source", "github"]);
    assert_eq!(stable[3].args, ["-I", "--source", "mirrorc"]);
    assert!(stable[3].requires_cdk);

    let alpha = updater_launch_options(UpdateChannel::Alpha);
    assert_eq!(
        alpha.iter().map(|option| option.source).collect::<Vec<_>>(),
        vec![UpdaterSource::DfsAlpha, UpdaterSource::MirrorChyanAlpha]
    );
    assert_eq!(alpha[0].args, ["-I", "--source", "dfs-alpha"]);
    assert_eq!(alpha[1].args, ["-I", "--source", "mirrorc-alpha"]);
}

#[test]
fn updater_launch_plan_parses_legacy_aliases_and_rejects_unknown_sources() {
    assert_eq!(
        updater_launch_plan(None).unwrap().source,
        UpdaterSource::Default
    );
    assert_eq!(updater_launch_plan(Some("default")).unwrap().args, ["-I"]);
    assert_eq!(
        updater_launch_plan(Some("steambird-alpha"))
            .unwrap()
            .source_arg,
        Some("dfs-alpha")
    );
    assert_eq!(
        updater_launch_plan(Some("mirror-chyan-alpha"))
            .unwrap()
            .source,
        UpdaterSource::MirrorChyanAlpha
    );
    assert!(updater_launch_plan(Some("unknown")).is_err());
}

#[test]
fn version_comparison_returns_false_on_invalid_versions() {
    assert!(is_new_version("1.0.0", "1.0.1"));
    assert!(!is_new_version("1.0.1", "1.0.0"));
    assert!(!is_new_version("dev", "1.0.0"));
    assert!(!is_new_version("1.0.0", "dev"));
}

#[test]
fn redeem_code_feed_update_decision_matches_legacy_numeric_compare() {
    let update = redeem_code_feed_update_decision("20251013", Some("20251014"));
    assert!(update.has_update);
    assert_eq!(update.remote_version, Some("20251014".to_string()));
    assert_eq!(update.request_url, REDEEM_CODE_UPDATE_TIME_URL);

    assert!(!redeem_code_feed_update_decision("20251013", Some("20251013")).has_update);
    assert!(!redeem_code_feed_update_decision("20251013", Some("abc")).has_update);
    assert!(!redeem_code_feed_update_decision("abc", Some("20251014")).has_update);
    assert!(!redeem_code_feed_update_decision("20251013", None).has_update);
}

#[test]
fn redeem_code_feed_items_deserialize_legacy_pascal_case_json() {
    let items = parse_redeem_code_feed_items(
        r#"[
          {
            "Title": "前瞻直播兑换码",
            "Content": "原石 * 300",
            "Time": "2026-06-20 20:00",
            "Tag": "前瞻",
            "Valid": "2026-06-21",
            "Codes": ["GENSHINGIFT", "ABC123"]
          }
        ]"#,
    )
    .expect("feed json should parse");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "前瞻直播兑换码");
    assert_eq!(items[0].codes, ["GENSHINGIFT", "ABC123"]);
    assert_eq!(items[0].tag, "前瞻");
}

#[test]
fn redeem_code_live_act_id_is_extracted_from_legacy_bbs_shape() {
    let structured = serde_json::json!([
        { "insert": "无关文本", "attributes": { "link": "" } },
        {
            "insert": "观看直播",
            "attributes": {
                "link": "https://webstatic.mihoyo.com/event?act_id=e20260620preview&utm=1"
            }
        }
    ])
    .to_string();
    let response = serde_json::json!({
        "retcode": 0,
        "data": {
            "list": [
                {
                    "post": {
                        "post": {
                            "subject": "原神前瞻特别节目",
                            "structured_content": structured
                        }
                    }
                }
            ]
        }
    });

    assert_eq!(
        redeem_code_live_act_id_from_bbs_response(&response.to_string()).as_deref(),
        Some("e20260620preview")
    );
}

#[test]
fn redeem_code_live_index_extracts_code_version_and_title() {
    let response = serde_json::json!({
        "retcode": 0,
        "data": {
            "live": {
                "code_ver": "v5",
                "title": "5.0 前瞻特别节目"
            }
        }
    });

    assert_eq!(
        redeem_code_live_index_from_response(&response.to_string()),
        Some(("v5".to_string(), "5.0 前瞻特别节目".to_string()))
    );
}

#[test]
fn redeem_code_live_codes_strip_html_titles() {
    let response = serde_json::json!({
        "retcode": 0,
        "data": {
            "code_list": [
                { "title": "<b>原石</b> * 100", "code": "LIVE100" },
                { "title": "摩拉", "code": "LIVE200" },
                { "title": "empty", "code": "" }
            ]
        }
    });

    let codes = redeem_code_live_codes_from_response(&response.to_string());
    assert_eq!(
        codes,
        vec![
            RedeemCodeLiveCode {
                code: "LIVE100".to_string(),
                items: "原石 * 100".to_string(),
            },
            RedeemCodeLiveCode {
                code: "LIVE200".to_string(),
                items: "摩拉".to_string(),
            },
        ]
    );
}
