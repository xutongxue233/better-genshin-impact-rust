use super::*;
use crate::GenshinAction;
use std::fs;

#[test]
fn deserializes_existing_snake_case_pathing_json() {
    let json = r#"{
        "info": {
            "name": "sample",
            "type": "collect",
            "map_name": "Teyvat",
            "bgi_version": "0.45.0"
        },
        "positions": [
            {
                "id": 1,
                "x": 4527.51,
                "y": 4825.12,
                "type": "path",
                "move_mode": "dash",
                "action": "",
                "action_params": ""
            },
            {
                "x": 4517.81,
                "y": 4866.36,
                "action": "pick_around"
            }
        ]
    }"#;

    let task = PathingTask::from_json(json).unwrap();
    assert_eq!(task.info.name, "sample");
    assert_eq!(task.positions.len(), 2);
    assert!(task.has_action("pick_around"));
    assert_eq!(task.summary().waypoint_count, 2);
}

#[test]
fn merges_control_json_like_legacy_json_merger() {
    let control: Value = json5::from_str(
        r#"{
            global_cover: { config: { realtime_triggers: { AutoPick: false } } },
            json_list: [
                { name: "route", cover: { info: { author: "tester" } } }
            ]
        }"#,
    )
    .unwrap();
    let mut original: Value = json5::from_str(
        r#"{
            info: { name: "route", type: "collect" },
            config: { realtime_triggers: { AutoPick: true } },
            positions: []
        }"#,
    )
    .unwrap();

    merge_pathing_json(&control, &mut original, "route");
    assert_eq!(original["config"]["realtime_triggers"]["AutoPick"], false);
    assert_eq!(original["info"]["author"], "tester");
}

#[test]
fn file_pathing_task_propagates_monster_loot_split_to_waypoints() {
    let root = test_root("pathing-monster-loot-split");
    let path = root.join("route.json");
    fs::write(
        &path,
        r#"{
            "info": { "name": "route", "enable_monster_loot_split": true },
            "positions": [
                {
                    "x": 1.0,
                    "y": 2.0,
                    "point_ext_params": { "enable_monster_loot_split": false }
                },
                { "x": 3.0, "y": 4.0 }
            ]
        }"#,
    )
    .unwrap();

    let task = read_pathing_task(&path).unwrap();

    assert_eq!(task.file_name.as_deref(), Some("route.json"));
    assert_eq!(task.full_path.as_deref(), Some(path.as_path()));
    assert!(task
        .positions
        .iter()
        .all(|position| position.point_ext_params.enable_monster_loot_split));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn pathing_execution_plan_splits_segments_like_legacy_executor() {
    let task = PathingTask::from_json(
        r#"{
            "info": {
                "name": "route",
                "type": "farming",
                "map_name": "Teyvat",
                "map_match_method": "featureMatch"
            },
            "config": { "realtime_triggers": { "AutoPick": true } },
            "farming_info": {
                "allow_farming_count": true,
                "primary_target": "elite",
                "elite_mob_count": 2
            },
            "positions": [
                { "x": 1.0, "y": 2.0, "type": "path", "move_mode": "dash" },
                { "x": 3.0, "y": 4.0, "type": "target", "action": "fight" },
                { "x": 5.0, "y": 6.0, "type": "teleport" },
                { "x": 7.0, "y": 8.0, "type": "orientation", "action": "log_output" },
                { "x": 9.0, "y": 10.0, "type": "path", "action": "up_down_grab_leaf" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();

    assert_eq!(plan.map_match_method.as_deref(), Some("featureMatch"));
    assert_eq!(plan.retry_times, 2);
    assert!(plan.has_positions);
    assert!(plan.preflight.switch_party_before);
    assert!(plan.preflight.validate_game_with_task);
    assert!(plan.preflight.initialize_pathing);
    assert!(plan.preflight.update_current_pathing);
    assert!(plan.preflight.require_16_by_9_resolution);
    assert_eq!(plan.preflight.minimum_width, 1920);
    assert_eq!(plan.preflight.minimum_height, 1080);
    assert!(plan.preflight.convert_waypoints_for_track);
    assert_eq!(plan.preflight.delay_before_warm_up_ms, 100);
    assert!(plan.preflight.warm_up_navigation);
    assert!(plan.autopick_realtime_trigger_enabled);
    assert_eq!(plan.segment_count, 2);
    assert_eq!(plan.waypoint_count, 5);
    assert_eq!(plan.action_count, 3);
    assert_eq!(plan.expected_fight_count, 1);
    assert!(plan.farming.allow_farming_count);
    assert_eq!(plan.farming.primary_target, "elite");
    assert_eq!(plan.farming.elite_mob_count, 2.0);
    assert_eq!(plan.movement_contract.contract_version, 1);
    assert!(!plan.movement_contract.movement_executor_ready);
    assert!(!plan.movement_contract.native_pathing_completed);
    assert_eq!(plan.movement_contract.map_name, "Teyvat");
    assert_eq!(
        plan.movement_contract.map_match_method.as_deref(),
        Some("featureMatch")
    );
    assert_eq!(plan.movement_contract.segment_count, 2);
    assert_eq!(plan.movement_contract.waypoint_count, 5);
    assert!(plan.movement_contract.release_input_after_segment_attempt);
    assert!(plan
        .movement_contract
        .pending_dependencies
        .contains(&PathingMovementDependency::CoordinateConversion));
    assert!(plan
        .movement_contract
        .pending_dependencies
        .contains(&PathingMovementDependency::InputDispatch));

    assert_eq!(plan.segments[0].waypoint_count, 2);
    assert!(!plan.segments[0].starts_with_teleport);
    assert_eq!(
        plan.segments[0].seed_previous_position,
        Some(PathingPoint { x: 1.0, y: 2.0 })
    );
    assert_eq!(
        plan.segments[0].seed_previous_position_coordinate_space,
        Some(PathingCoordinateSpace::RouteJson)
    );
    assert!(plan.segments[0].seed_previous_position_requires_track_conversion);
    assert_eq!(plan.segments[1].waypoint_count, 3);
    assert!(plan.segments[1].starts_with_teleport);
    assert_eq!(plan.segments[1].seed_previous_position, None);
    assert_eq!(
        plan.segments[1].seed_previous_position_coordinate_space,
        None
    );
    assert!(!plan.segments[1].seed_previous_position_requires_track_conversion);

    let target = &plan.segments[0].waypoints[1];
    assert_eq!(target.global_index, 1);
    assert_eq!(target.route_point, PathingPoint { x: 3.0, y: 4.0 });
    assert_eq!(target.track_point, None);
    assert!(target.track_conversion_pending);
    assert_eq!(
        target.declared_action_use,
        Some(PathingActionUseWaypointType::Path)
    );
    assert!(target.effective_target_point);
    assert!(target.phases.contains(&PathingWaypointPhase::MoveCloseTo));
    assert!(target.phases.contains(&PathingWaypointPhase::RunAction));
    let target_contract = &plan.movement_contract.segments[0].waypoints[1];
    assert_eq!(target_contract.route_point, PathingPoint { x: 3.0, y: 4.0 });
    assert_eq!(target_contract.track_point, None);
    assert!(target_contract.track_conversion_pending);
    let before_move_contract = target_contract
        .phase_contracts
        .iter()
        .find(|contract| contract.phase == PathingWaypointPhase::BeforeMoveToTarget)
        .unwrap();
    assert_eq!(
        before_move_contract.native_status,
        PathingNativePhaseStatus::ReadyByRuntime
    );
    assert!(before_move_contract.pending_dependencies.is_empty());
    let move_close_contract = target_contract
        .phase_contracts
        .iter()
        .find(|contract| contract.phase == PathingWaypointPhase::MoveCloseTo)
        .unwrap();
    assert_eq!(
        move_close_contract.target_point,
        Some(PathingPoint { x: 3.0, y: 4.0 })
    );
    assert_eq!(
        move_close_contract.coordinate_space,
        Some(PathingCoordinateSpace::RouteJson)
    );
    assert!(move_close_contract.requires_track_conversion);
    assert_eq!(
        move_close_contract.native_status,
        PathingNativePhaseStatus::Pending
    );
    assert!(move_close_contract
        .pending_dependencies
        .contains(&PathingMovementDependency::MovementTermination));

    let teleport = &plan.segments[1].waypoints[0];
    assert_eq!(
        teleport.phases,
        vec![
            PathingWaypointPhase::RecoverWhenLowHp,
            PathingWaypointPhase::HandleTeleport
        ]
    );
    let teleport_contract = &plan.movement_contract.segments[1].waypoints[0];
    assert!(teleport_contract
        .phase_contracts
        .iter()
        .any(
            |contract| contract.phase == PathingWaypointPhase::HandleTeleport
                && contract
                    .pending_dependencies
                    .contains(&PathingMovementDependency::Teleport)
        ));

    let orientation = &plan.segments[1].waypoints[1];
    assert!(orientation.phases.contains(&PathingWaypointPhase::FaceTo));
    assert!(!orientation
        .phases
        .contains(&PathingWaypointPhase::MoveCloseTo));

    let leaf = &plan.segments[1].waypoints[2];
    assert!(!leaf.phases.contains(&PathingWaypointPhase::MoveTo));
    assert!(!leaf.phases.contains(&PathingWaypointPhase::MoveCloseTo));
    assert!(leaf.phases.contains(&PathingWaypointPhase::RunAction));
}

fn test_root(name: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("bgi-core-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn pathing_execution_plan_preserves_linnea_mining_action_params_and_rules() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "mine-route", "type": "mining", "map_name": "Teyvat" },
            "positions": [
                { "x": 10.0, "y": 20.0, "type": "path", "action": "linnea_mining", "action_params": "mines=3, rounds=2" },
                { "x": 11.0, "y": 21.0, "type": "path", "action": "linnea_mining", "action_params": "5" },
                { "x": 12.0, "y": 22.0, "type": "path", "action": "linnea_mining", "action_params": "0,1000" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();

    assert_eq!(plan.action_count, 3);
    let first = &plan.segments[0].waypoints[0];
    assert_eq!(
        first.declared_action_use,
        Some(PathingActionUseWaypointType::Custom)
    );
    assert!(first.phases.contains(&PathingWaypointPhase::RunAction));
    let Some(PathingActionPlan::LinneaMining(mining)) = &first.action_plan else {
        panic!("expected linnea mining action plan");
    };
    assert_eq!(mining.action_code, "linnea_mining");
    assert_eq!(mining.raw_params.as_deref(), Some("mines=3, rounds=2"));
    assert_eq!(mining.mine_count, 3);
    assert_eq!(mining.scan_rounds, 3);
    assert!(mining.prefer_right);
    assert_eq!(mining.avatar_name, "莉奈娅");
    assert!(mining.switch_avatar_before_mining);
    assert_eq!(mining.switch_wait_ms, 500);
    assert_eq!(mining.aiming_mode_action, GenshinAction::SwitchAimingMode);
    assert_eq!(mining.enter_aim_wait_ms, 400);
    assert_eq!(mining.detection_rule.model_name, "BgiMine");
    assert_eq!(
        mining.detection_rule.model_relative_path,
        "Assets/Model/Mine/bgi_mine.onnx"
    );
    assert_eq!(mining.detection_rule.accepted_label, "ore");
    assert_eq!(mining.detection_rule.confidence_threshold, 0.70);
    assert_eq!(
        mining.detection_rule.source,
        LinneaMiningDetectionSource::FullCapture
    );
    assert_eq!(mining.cluster_rule.base_cluster_distance_1080p, 400.0);
    assert_eq!(mining.cluster_rule.base_cluster_area_1080p, 1_800.0);
    assert_eq!(mining.cluster_rule.base_alignment_expansion_1080p, 3.0);
    assert_eq!(mining.cluster_rule.base_edge_ignore_1080p, 200.0);
    assert_eq!(mining.cluster_rule.area_ratio_threshold, 4.0);
    assert!(mining.cluster_rule.prefer_right_when_scan_rounds_gt_one);
    assert_eq!(mining.alignment_rule.max_inner_retry, 7);
    assert_eq!(mining.alignment_rule.element_sight_refresh_ms, 3_000);
    assert_eq!(mining.alignment_rule.refresh_release_ms, 100);
    assert_eq!(mining.alignment_rule.refresh_hold_ms, 1_500);
    assert_eq!(mining.alignment_rule.aim_sensitivity_factor_x, 0.45);
    assert_eq!(mining.alignment_rule.aim_sensitivity_factor_y, 0.80);
    assert_eq!(mining.alignment_rule.aim_move_delay_ms, 150);
    assert!(
        mining
            .alignment_rule
            .fallback_shot_on_last_successful_detection
    );
    assert_eq!(mining.scan_rule.middle_button_hold_ms, 1_500);
    assert_eq!(mining.scan_rule.middle_button_release_ms, 300);
    assert_eq!(mining.scan_rule.compensate_detection_hold_ms, 1_500);
    assert_eq!(mining.scan_rule.compensate_move_wait_ms, 800);
    assert_eq!(mining.scan_rule.left_turn_step_1080p, -250);
    assert_eq!(mining.scan_rule.left_turn_wait_ms, 800);
    assert_eq!(mining.mine_rule.compensate_up_pixels, -25);
    assert_eq!(mining.mine_rule.compensate_up_wait_ms, 10);
    assert_eq!(mining.mine_rule.attack_button, "LeftMouse");
    assert_eq!(mining.mine_rule.after_attack_wait_ms, 2_000);
    assert_eq!(
        mining.cleanup_rule.leave_aiming_mode_action,
        GenshinAction::SwitchAimingMode
    );
    assert!(mining.cleanup_rule.middle_button_up);
    assert!(mining.cleanup_rule.clear_vision_drawings);
    assert!(!mining.executor_ready);

    let Some(PathingActionPlan::LinneaMining(single_number)) =
        &plan.segments[0].waypoints[1].action_plan
    else {
        panic!("expected linnea mining action plan");
    };
    assert_eq!(single_number.mine_count, 5);
    assert_eq!(single_number.scan_rounds, 5);

    let Some(PathingActionPlan::LinneaMining(clamped)) = &plan.segments[0].waypoints[2].action_plan
    else {
        panic!("expected linnea mining action plan");
    };
    assert_eq!(clamped.mine_count, 1);
    assert_eq!(clamped.scan_rounds, 999);
}

#[test]
fn pathing_execution_plan_parses_set_time_action_params_like_legacy_handler() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "time-route", "type": "collect", "map_name": "Teyvat" },
            "positions": [
                { "x": 10.0, "y": 20.0, "type": "path", "action": "set_time", "action_params": "7:30" },
                { "x": 11.0, "y": 21.0, "type": "path", "action": "set_time", "action_params": "19:45:false" },
                { "x": 12.0, "y": 22.0, "type": "path", "action": "set_time", "action_params": "bad" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();

    let first = &plan.segments[0].waypoints[0];
    assert_eq!(
        first.declared_action_use,
        Some(PathingActionUseWaypointType::Custom)
    );
    assert!(first.phases.contains(&PathingWaypointPhase::RunAction));
    let Some(PathingActionPlan::SetTime(set_time)) = &first.action_plan else {
        panic!("expected set_time action plan");
    };
    assert_eq!(set_time.action_code, "set_time");
    assert_eq!(set_time.raw_params.as_deref(), Some("7:30"));
    assert_eq!(set_time.common_job_task_key, "SetTime");
    assert_eq!(set_time.hour, Some(7));
    assert_eq!(set_time.minute, Some(30));
    assert_eq!(set_time.skip_time_adjustment_animation, Some(true));
    assert_eq!(set_time.parse_error, None);
    assert!(set_time.executor_ready);

    let Some(PathingActionPlan::SetTime(explicit_no_skip)) =
        &plan.segments[0].waypoints[1].action_plan
    else {
        panic!("expected set_time action plan");
    };
    assert_eq!(explicit_no_skip.hour, Some(19));
    assert_eq!(explicit_no_skip.minute, Some(45));
    assert_eq!(explicit_no_skip.skip_time_adjustment_animation, Some(false));
    assert!(explicit_no_skip.executor_ready);

    let Some(PathingActionPlan::SetTime(invalid)) = &plan.segments[0].waypoints[2].action_plan
    else {
        panic!("expected set_time action plan");
    };
    assert!(!invalid.executor_ready);
    assert!(invalid.parse_error.as_deref().unwrap().contains("HH:mm"));
}

#[test]
fn pathing_execution_plan_models_log_and_common_job_actions() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "action-route", "type": "collect", "map_name": "Teyvat" },
            "positions": [
                { "x": 10.0, "y": 20.0, "type": "orientation", "action": "log_output", "action_params": "arrived" },
                { "x": 11.0, "y": 21.0, "type": "path", "action": "exit_and_relogin" },
                { "x": 12.0, "y": 22.0, "type": "path", "action": "wonderland_cycle", "action_params": "ignored" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();

    let Some(PathingActionPlan::LogOutput(log_output)) = &plan.segments[0].waypoints[0].action_plan
    else {
        panic!("expected log_output action plan");
    };
    assert_eq!(log_output.action_code, "log_output");
    assert_eq!(log_output.message, "arrived");
    assert!(log_output.executor_ready);

    let Some(PathingActionPlan::CommonJob(relogin)) = &plan.segments[0].waypoints[1].action_plan
    else {
        panic!("expected exit_and_relogin common-job action plan");
    };
    assert_eq!(relogin.action_code, "exit_and_relogin");
    assert_eq!(relogin.common_job_task_key, "Relogin");
    assert!(relogin.executor_ready);

    let Some(PathingActionPlan::CommonJob(wonderland)) = &plan.segments[0].waypoints[2].action_plan
    else {
        panic!("expected wonderland_cycle common-job action plan");
    };
    assert_eq!(wonderland.action_code, "wonderland_cycle");
    assert_eq!(wonderland.raw_params.as_deref(), Some("ignored"));
    assert_eq!(wonderland.common_job_task_key, "WonderlandCycle");
    assert!(wonderland.executor_ready);
}

#[test]
fn empty_pathing_execution_plan_skips_preflight_like_legacy_executor() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "empty", "type": "collect", "map_name": "Teyvat" },
            "positions": []
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();

    assert!(!plan.has_positions);
    assert_eq!(plan.segment_count, 0);
    assert_eq!(plan.waypoint_count, 0);
    assert!(!plan.preflight.switch_party_before);
    assert!(!plan.preflight.validate_game_with_task);
    assert!(!plan.preflight.initialize_pathing);
    assert!(!plan.preflight.update_current_pathing);
    assert!(!plan.preflight.require_16_by_9_resolution);
    assert!(!plan.preflight.convert_waypoints_for_track);
    assert_eq!(plan.preflight.delay_before_warm_up_ms, 0);
    assert!(!plan.preflight.warm_up_navigation);
    assert!(!plan.movement_contract.movement_executor_ready);
    assert!(!plan.movement_contract.native_pathing_completed);
    assert!(plan.movement_contract.pending_dependencies.is_empty());
    assert_eq!(plan.movement_contract.segment_count, 0);
    assert_eq!(plan.movement_contract.waypoint_count, 0);
    assert_eq!(plan.movement_contract.segments.len(), 0);
    assert_eq!(plan.segments, Vec::<PathingSegmentPlan>::new());
}
