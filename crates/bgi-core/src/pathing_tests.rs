use super::*;
use crate::{GenshinAction, KeyId};
use std::fs;

#[test]
fn legacy_track_map_point_converts_teyvat_route_coordinates() {
    assert_eq!(
        legacy_track_map_point("Teyvat", PathingPoint { x: 1.0, y: 2.0 }),
        Some(PathingPoint {
            x: 32766.0,
            y: 16380.0,
        })
    );
    assert_eq!(
        legacy_track_map_point("teyvat", PathingPoint { x: -5.5, y: 7.25 }),
        Some(PathingPoint {
            x: 32779.0,
            y: 16369.5,
        })
    );
    assert_eq!(
        legacy_track_map_point("Enkanomiya", PathingPoint { x: 1.0, y: 2.0 }),
        None
    );
}

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
    assert_eq!(plan.segments[1].pre_teleport_delay_ms, 0);
    assert_eq!(plan.movement_contract.segments[1].pre_teleport_delay_ms, 0);
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

#[test]
fn pathing_execution_plan_preserves_legacy_pre_teleport_delay_between_segments() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "route", "type": "collect", "map_name": "Teyvat" },
            "positions": [
                { "x": 1.0, "y": 2.0, "type": "path", "move_mode": "dash" },
                { "x": 3.0, "y": 4.0, "type": "target", "action": "pick_up_collect" },
                { "x": 5.0, "y": 6.0, "type": "teleport" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();

    assert_eq!(plan.segment_count, 2);
    assert_eq!(plan.segments[0].pre_teleport_delay_ms, 0);
    assert_eq!(plan.segments[1].pre_teleport_delay_ms, 1_000);
    assert_eq!(
        plan.movement_contract.segments[1].pre_teleport_delay_ms,
        1_000
    );
}

#[test]
fn pathing_execution_plan_skips_pre_teleport_delay_after_legacy_fast_endpoints() {
    for (waypoint_type, action) in [
        ("teleport", None),
        ("target", Some("fight")),
        ("target", Some("nahida_collect")),
        ("target", Some("pick_around")),
    ] {
        let action_json = action
            .map(|action| format!(r#", "action": "{action}""#))
            .unwrap_or_default();
        let json = format!(
            r#"{{
                "info": {{ "name": "route", "type": "collect", "map_name": "Teyvat" }},
                "positions": [
                    {{ "x": 1.0, "y": 2.0, "type": "{waypoint_type}"{action_json} }},
                    {{ "x": 3.0, "y": 4.0, "type": "teleport" }}
                ]
            }}"#
        );
        let task = PathingTask::from_json(&json).unwrap();

        let plan = task.execution_plan();

        assert_eq!(plan.segment_count, 2);
        assert_eq!(plan.segments[1].pre_teleport_delay_ms, 0);
        assert_eq!(plan.movement_contract.segments[1].pre_teleport_delay_ms, 0);
    }
}

#[test]
fn pathing_execution_plan_applies_track_conversion_to_waypoints_and_segment_seed() {
    let task = PathingTask::from_json(
        r#"{
            "info": {
                "name": "route",
                "type": "collect",
                "map_name": "Teyvat",
                "map_match_method": "featureMatch"
            },
            "positions": [
                { "x": 1.0, "y": 2.0, "type": "path", "move_mode": "dash" },
                { "x": 3.0, "y": 4.0, "type": "target", "action": "pick_up_collect" },
                { "x": 5.0, "y": 6.0, "type": "teleport" }
            ]
        }"#,
    )
    .unwrap();
    let mut contexts = Vec::new();

    let plan = task.execution_plan_with_track_converter(|context| {
        contexts.push((
            context.map_name.to_string(),
            context.map_match_method.map(str::to_string),
            context.global_index,
            context.segment_index,
            context.segment_waypoint_index,
            context.waypoint_type.to_string(),
            context.action.map(str::to_string),
        ));
        Some(PathingPoint {
            x: context.route_point.x + 1_000.0,
            y: context.route_point.y + 2_000.0,
        })
    });

    assert_eq!(
        contexts,
        vec![
            (
                "Teyvat".to_string(),
                Some("featureMatch".to_string()),
                0,
                0,
                0,
                "path".to_string(),
                None
            ),
            (
                "Teyvat".to_string(),
                Some("featureMatch".to_string()),
                1,
                0,
                1,
                "target".to_string(),
                Some("pick_up_collect".to_string())
            ),
            (
                "Teyvat".to_string(),
                Some("featureMatch".to_string()),
                2,
                1,
                0,
                "teleport".to_string(),
                None
            )
        ]
    );
    assert_eq!(
        plan.segments[0].seed_previous_position,
        Some(PathingPoint {
            x: 1_001.0,
            y: 2_002.0
        })
    );
    assert_eq!(
        plan.segments[0].seed_previous_position_coordinate_space,
        Some(PathingCoordinateSpace::LegacyTrackMap)
    );
    assert!(!plan.segments[0].seed_previous_position_requires_track_conversion);
    assert!(!plan
        .movement_contract
        .pending_dependencies
        .contains(&PathingMovementDependency::CoordinateConversion));
    assert!(plan
        .movement_contract
        .pending_dependencies
        .contains(&PathingMovementDependency::InputDispatch));
    assert!(plan
        .movement_contract
        .pending_dependencies
        .contains(&PathingMovementDependency::MovementTermination));

    let target = &plan.segments[0].waypoints[1];
    assert_eq!(
        target.track_point,
        Some(PathingPoint {
            x: 1_003.0,
            y: 2_004.0
        })
    );
    assert!(!target.track_conversion_pending);
    let target_contract = &plan.movement_contract.segments[0].waypoints[1];
    assert_eq!(target_contract.track_point, target.track_point);
    assert!(!target_contract.track_conversion_pending);
    let move_close_contract = target_contract
        .phase_contracts
        .iter()
        .find(|contract| contract.phase == PathingWaypointPhase::MoveCloseTo)
        .unwrap();
    assert_eq!(move_close_contract.target_point, target.track_point);
    assert_eq!(
        move_close_contract.coordinate_space,
        Some(PathingCoordinateSpace::LegacyTrackMap)
    );
    assert!(!move_close_contract.requires_track_conversion);
    assert!(!move_close_contract
        .pending_dependencies
        .contains(&PathingMovementDependency::CoordinateConversion));
    assert!(move_close_contract
        .pending_dependencies
        .contains(&PathingMovementDependency::InputDispatch));
}

#[test]
fn pathing_execution_plan_keeps_coordinate_conversion_pending_when_converter_declines_point() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "route", "type": "collect", "map_name": "Teyvat" },
            "positions": [
                { "x": 1.0, "y": 2.0, "type": "path", "move_mode": "dash" },
                { "x": 3.0, "y": 4.0, "type": "target", "action": "pick_up_collect" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan_with_track_converter(|context| {
        (context.global_index == 0).then_some(PathingPoint { x: 10.0, y: 20.0 })
    });

    assert_eq!(
        plan.segments[0].seed_previous_position,
        Some(PathingPoint { x: 10.0, y: 20.0 })
    );
    assert_eq!(
        plan.segments[0].seed_previous_position_coordinate_space,
        Some(PathingCoordinateSpace::LegacyTrackMap)
    );
    let declined = &plan.movement_contract.segments[0].waypoints[1];
    assert_eq!(declined.track_point, None);
    assert!(declined.track_conversion_pending);
    let move_close_contract = declined
        .phase_contracts
        .iter()
        .find(|contract| contract.phase == PathingWaypointPhase::MoveCloseTo)
        .unwrap();
    assert_eq!(
        move_close_contract.coordinate_space,
        Some(PathingCoordinateSpace::RouteJson)
    );
    assert!(move_close_contract.requires_track_conversion);
    assert!(move_close_contract
        .pending_dependencies
        .contains(&PathingMovementDependency::CoordinateConversion));
    assert!(plan
        .movement_contract
        .pending_dependencies
        .contains(&PathingMovementDependency::CoordinateConversion));
}

#[test]
fn pathing_movement_contract_aggregates_present_phase_dependencies_only() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "route", "type": "collect", "map_name": "Teyvat" },
            "positions": [
                { "x": 1.0, "y": 2.0, "type": "path", "move_mode": "dash" },
                { "x": 3.0, "y": 4.0, "type": "orientation", "action": "log_output", "action_params": "turn" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();
    let dependencies = &plan.movement_contract.pending_dependencies;

    assert!(!plan.movement_contract.movement_executor_ready);
    assert!(dependencies.contains(&PathingMovementDependency::CoordinateConversion));
    assert!(dependencies.contains(&PathingMovementDependency::PositionObservation));
    assert!(dependencies.contains(&PathingMovementDependency::CameraRotation));
    assert!(dependencies.contains(&PathingMovementDependency::InputDispatch));
    assert!(dependencies.contains(&PathingMovementDependency::LowHpRecovery));
    assert!(dependencies.contains(&PathingMovementDependency::TrapEscape));
    assert!(dependencies.contains(&PathingMovementDependency::MovementTermination));
    assert!(!dependencies.contains(&PathingMovementDependency::Teleport));
    assert!(!dependencies.contains(&PathingMovementDependency::ActionHandlers));

    let orientation_contract = &plan.movement_contract.segments[0].waypoints[1];
    let run_action_contract = orientation_contract
        .phase_contracts
        .iter()
        .find(|contract| contract.phase == PathingWaypointPhase::RunAction)
        .expect("log_output waypoint should keep a run-action phase");
    assert_eq!(
        run_action_contract.native_status,
        PathingNativePhaseStatus::Pending
    );
    assert!(run_action_contract.pending_dependencies.is_empty());
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
fn pathing_execution_plan_models_force_tp_as_teleport_intent() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "force teleport route", "type": "collect", "map_name": "Teyvat" },
            "positions": [
                { "x": 9282.7, "y": -2163.58, "type": "teleport", "move_mode": "walk", "action": "force_tp", "action_params": "" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();
    let waypoint = &plan.segments[0].waypoints[0];

    assert_eq!(
        waypoint.declared_action_use,
        Some(PathingActionUseWaypointType::Custom)
    );
    assert_eq!(
        waypoint.phases,
        vec![
            PathingWaypointPhase::RecoverWhenLowHp,
            PathingWaypointPhase::HandleTeleport
        ]
    );
    assert!(!waypoint.phases.contains(&PathingWaypointPhase::RunAction));

    let Some(PathingActionPlan::ForceTeleport(force_tp)) = &waypoint.action_plan else {
        panic!("expected force_tp teleport intent plan");
    };
    assert_eq!(force_tp.action_code, "force_tp");
    assert_eq!(force_tp.raw_params.as_deref(), Some(""));
    assert!(force_tp.force_teleport);
    assert!(!force_tp.executor_ready);
    assert!(force_tp.notes.contains("HandleTeleport"));

    let teleport_contract = &plan.movement_contract.segments[0].waypoints[0];
    let handle_teleport_contract = teleport_contract
        .phase_contracts
        .iter()
        .find(|contract| contract.phase == PathingWaypointPhase::HandleTeleport)
        .expect("expected HandleTeleport contract");
    assert_eq!(
        handle_teleport_contract.target_point,
        Some(PathingPoint {
            x: 9282.7,
            y: -2163.58
        })
    );
    assert!(handle_teleport_contract
        .pending_dependencies
        .contains(&PathingMovementDependency::Teleport));
}

#[test]
fn pathing_execution_plan_models_use_gadget_not_wait_slice() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "gadget route", "type": "collect", "map_name": "Teyvat" },
            "positions": [
                { "x": 10.0, "y": 20.0, "type": "path", "action": "use_gadget", "action_params": "not_wait" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();
    let waypoint = &plan.segments[0].waypoints[0];

    assert_eq!(
        waypoint.declared_action_use,
        Some(PathingActionUseWaypointType::Custom)
    );
    assert!(waypoint.phases.contains(&PathingWaypointPhase::RunAction));
    let Some(PathingActionPlan::UseGadget(use_gadget)) = &waypoint.action_plan else {
        panic!("expected use_gadget action plan");
    };
    assert_eq!(use_gadget.action_code, "use_gadget");
    assert_eq!(use_gadget.raw_params.as_deref(), Some("not_wait"));
    assert_eq!(use_gadget.genshin_action, GenshinAction::QuickUseGadget);
    assert!(use_gadget.not_wait);
    assert!(!use_gadget.cooldown_ocr_required);
    assert_eq!(use_gadget.max_wait_seconds, None);
    assert_eq!(use_gadget.max_wait_parse_error, None);
    assert_eq!(use_gadget.quick_use_gadget_press_count, 2);
    assert_eq!(use_gadget.handler_delay_ms, 300);
    assert_eq!(use_gadget.path_executor_after_action_delay_ms, 1_000);
    assert!(use_gadget.executor_ready);

    let run_action_contract = plan.movement_contract.segments[0].waypoints[0]
        .phase_contracts
        .iter()
        .find(|contract| contract.phase == PathingWaypointPhase::RunAction)
        .expect("expected RunAction contract");
    assert_eq!(
        run_action_contract.pending_dependencies,
        vec![PathingMovementDependency::InputDispatch]
    );
}

#[test]
fn pathing_execution_plan_keeps_use_gadget_cooldown_branch_pending() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "gadget cd route", "type": "collect", "map_name": "Teyvat" },
            "positions": [
                { "x": 10.0, "y": 20.0, "type": "path", "action": "use_gadget", "action_params": "2.5" },
                { "x": 11.0, "y": 21.0, "type": "path", "action": "use_gadget", "action_params": "bad" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();

    let Some(PathingActionPlan::UseGadget(timed)) = &plan.segments[0].waypoints[0].action_plan
    else {
        panic!("expected timed use_gadget action plan");
    };
    assert!(!timed.not_wait);
    assert!(timed.cooldown_ocr_required);
    assert_eq!(timed.max_wait_seconds, Some(2.5));
    assert_eq!(timed.max_wait_parse_error, None);
    assert_eq!(timed.cooldown_overlong_skip_threshold_seconds, 100.0);
    assert_eq!(timed.cooldown_wait_padding_ms, 100);
    assert!(!timed.executor_ready);

    let Some(PathingActionPlan::UseGadget(invalid)) = &plan.segments[0].waypoints[1].action_plan
    else {
        panic!("expected invalid use_gadget action plan");
    };
    assert_eq!(invalid.max_wait_seconds, Some(0.0));
    assert!(invalid
        .max_wait_parse_error
        .as_ref()
        .unwrap()
        .contains("bad"));
    assert!(!invalid.executor_ready);

    for waypoint in &plan.movement_contract.segments[0].waypoints {
        let run_action_contract = waypoint
            .phase_contracts
            .iter()
            .find(|contract| contract.phase == PathingWaypointPhase::RunAction)
            .expect("expected RunAction contract");
        assert!(run_action_contract
            .pending_dependencies
            .contains(&PathingMovementDependency::ActionHandlers));
    }
}

#[test]
fn pathing_execution_plan_models_nahida_collect_scan_sequence() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "nahida route", "type": "collect", "map_name": "Teyvat" },
            "positions": [
                { "x": 10.0, "y": 20.0, "type": "target", "action": "nahida_collect", "action_params": "ignored" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();
    let waypoint = &plan.segments[0].waypoints[0];

    assert_eq!(
        waypoint.declared_action_use,
        Some(PathingActionUseWaypointType::Custom)
    );
    assert!(waypoint.phases.contains(&PathingWaypointPhase::RunAction));
    let Some(PathingActionPlan::NahidaCollect(nahida_collect)) = &waypoint.action_plan else {
        panic!("expected nahida_collect action plan");
    };
    assert_eq!(nahida_collect.action_code, "nahida_collect");
    assert_eq!(nahida_collect.raw_params.as_deref(), Some("ignored"));
    assert_eq!(nahida_collect.avatar_name, "纳西妲");
    assert!(nahida_collect.requires_combat_scenes);
    assert!(nahida_collect.switch_avatar_before_collect);
    assert!(nahida_collect.wait_skill_cooldown_before_collect);
    assert!(nahida_collect.update_skill_cooldown_after_collect);
    assert!(nahida_collect.dpi_scale_required);
    assert_eq!(nahida_collect.lower_view_mouse_move_y, 10_000);
    assert_eq!(
        nahida_collect.elemental_skill_action,
        GenshinAction::ElementalSkill
    );
    assert_eq!(nahida_collect.ground_scan_iterations, 15);
    assert_eq!(nahida_collect.ground_scan_move_x, 400);
    assert!(nahida_collect.ground_scan_move_x_dpi_scaled);
    assert_eq!(nahida_collect.ground_scan_move_y, 500);
    assert!(!nahida_collect.ground_scan_move_y_dpi_scaled);
    assert_eq!(nahida_collect.raised_scan_iterations, 60);
    assert_eq!(nahida_collect.raised_scan_initial_move_y, -30);
    assert_eq!(nahida_collect.raised_scan_y_adjust_before_iteration, 20);
    assert_eq!(nahida_collect.raised_scan_y_adjust_delta, -20);
    assert_eq!(nahida_collect.scan_step_delay_ms, 30);
    assert_eq!(nahida_collect.post_skill_release_cd_update_delay_ms, 200);
    assert_eq!(nahida_collect.after_collect_delay_ms, 800);
    assert_eq!(nahida_collect.restore_view_key, KeyId::MOUSE_MIDDLE_BUTTON);
    assert_eq!(nahida_collect.restore_view_delay_ms, 1_000);
    assert_eq!(nahida_collect.path_executor_after_action_delay_ms, 1_000);
    assert!(nahida_collect.skill_release_in_finally);
    assert!(!nahida_collect.executor_ready);
    assert_eq!(nahida_collect.steps.len(), 159);

    assert_eq!(
        nahida_collect.steps[0],
        PathingNahidaCollectStep::MouseMoveBy {
            dx: 0,
            dy: 10_000,
            dx_dpi_scaled: false,
            dy_dpi_scaled: false,
        }
    );
    assert_eq!(
        nahida_collect.steps[2],
        PathingNahidaCollectStep::GenshinAction {
            action: GenshinAction::ElementalSkill,
            press: PathingInputPress::KeyDown
        }
    );
    assert_eq!(
        nahida_collect.steps[4],
        PathingNahidaCollectStep::MouseMoveBy {
            dx: 400,
            dy: 500,
            dx_dpi_scaled: true,
            dy_dpi_scaled: false,
        }
    );
    assert_eq!(
        nahida_collect.steps[34],
        PathingNahidaCollectStep::MouseMoveBy {
            dx: 400,
            dy: -30,
            dx_dpi_scaled: true,
            dy_dpi_scaled: true,
        }
    );
    assert_eq!(
        nahida_collect.steps[72],
        PathingNahidaCollectStep::MouseMoveBy {
            dx: 400,
            dy: -50,
            dx_dpi_scaled: true,
            dy_dpi_scaled: true,
        }
    );
    assert_eq!(
        nahida_collect.steps[154],
        PathingNahidaCollectStep::GenshinAction {
            action: GenshinAction::ElementalSkill,
            press: PathingInputPress::KeyUp
        }
    );
    assert_eq!(
        nahida_collect.steps[157],
        PathingNahidaCollectStep::Key {
            key: KeyId::MOUSE_MIDDLE_BUTTON,
            press: PathingInputPress::KeyPress
        }
    );

    let run_action_contract = plan.movement_contract.segments[0].waypoints[0]
        .phase_contracts
        .iter()
        .find(|contract| contract.phase == PathingWaypointPhase::RunAction)
        .expect("expected RunAction contract");
    assert_eq!(
        run_action_contract.pending_dependencies,
        vec![
            PathingMovementDependency::InputDispatch,
            PathingMovementDependency::ActionHandlers
        ]
    );
}

#[test]
fn pathing_execution_plan_models_elemental_collect_avatar_tables() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "elemental route", "type": "collect", "map_name": "Teyvat" },
            "positions": [
                { "x": 10.0, "y": 20.0, "type": "target", "action": "hydro_collect", "action_params": "ignored" },
                { "x": 11.0, "y": 21.0, "type": "target", "action": "electro_collect" },
                { "x": 12.0, "y": 22.0, "type": "target", "action": "anemo_collect" },
                { "x": 13.0, "y": 23.0, "type": "target", "action": "pyro_collect" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();

    let Some(PathingActionPlan::ElementalCollect(hydro)) =
        &plan.segments[0].waypoints[0].action_plan
    else {
        panic!("expected hydro elemental collect plan");
    };
    assert_eq!(hydro.action_code, "hydro_collect");
    assert_eq!(hydro.raw_params.as_deref(), Some("ignored"));
    assert_eq!(hydro.element, PathingElementalCollectElement::Hydro);
    assert_eq!(hydro.element_chinese, "水");
    assert!(hydro.requires_combat_scenes);
    assert!(hydro.legacy_preflight_validates_avatar);
    assert!(hydro.scans_team_order);
    assert!(hydro.switch_avatar_before_collect);
    assert!(hydro.normal_attack_preferred_over_skill);
    assert_eq!(hydro.normal_attack_duration_ms, 100);
    assert!(hydro.wait_skill_cooldown_before_skill);
    assert_eq!(hydro.path_executor_after_action_delay_ms, 1_000);
    assert!(!hydro.executor_ready);
    assert_eq!(hydro.candidates.len(), 10);
    assert_eq!(hydro.candidates[0].avatar_name, "芭芭拉");
    assert!(hydro.candidates[0].normal_attack);
    assert!(hydro.candidates[0].elemental_skill);
    assert_eq!(
        hydro.candidates[0].selected_action,
        PathingElementalCollectAvatarAction::NormalAttack
    );
    assert_eq!(hydro.candidates[0].normal_attack_duration_ms, Some(100));
    assert!(!hydro.candidates[0].waits_skill_cooldown);
    let nilou = hydro
        .candidates
        .iter()
        .find(|candidate| candidate.avatar_name == "妮露")
        .expect("expected Nilou hydro candidate");
    assert_eq!(
        nilou.selected_action,
        PathingElementalCollectAvatarAction::ElementalSkill
    );
    assert_eq!(nilou.normal_attack_duration_ms, None);
    assert!(nilou.waits_skill_cooldown);

    let Some(PathingActionPlan::ElementalCollect(electro)) =
        &plan.segments[0].waypoints[1].action_plan
    else {
        panic!("expected electro elemental collect plan");
    };
    assert_eq!(electro.element, PathingElementalCollectElement::Electro);
    assert_eq!(electro.element_chinese, "雷");
    assert_eq!(electro.candidates.len(), 8);
    assert!(electro.legacy_preflight_validates_avatar);

    let Some(PathingActionPlan::ElementalCollect(anemo)) =
        &plan.segments[0].waypoints[2].action_plan
    else {
        panic!("expected anemo elemental collect plan");
    };
    assert_eq!(anemo.element, PathingElementalCollectElement::Anemo);
    assert_eq!(anemo.element_chinese, "风");
    assert_eq!(anemo.candidates.len(), 11);
    assert!(anemo.legacy_preflight_validates_avatar);

    let Some(PathingActionPlan::ElementalCollect(pyro)) =
        &plan.segments[0].waypoints[3].action_plan
    else {
        panic!("expected pyro elemental collect plan");
    };
    assert_eq!(pyro.element, PathingElementalCollectElement::Pyro);
    assert_eq!(pyro.element_chinese, "火");
    assert_eq!(pyro.candidates.len(), 12);
    assert!(!pyro.legacy_preflight_validates_avatar);

    for (waypoint_index, waypoint) in plan.segments[0].waypoints.iter().enumerate() {
        assert_eq!(
            waypoint.declared_action_use,
            Some(PathingActionUseWaypointType::Target)
        );
        assert!(waypoint.phases.contains(&PathingWaypointPhase::RunAction));
        let run_action_contract = plan.movement_contract.segments[0].waypoints[waypoint_index]
            .phase_contracts
            .iter()
            .find(|contract| contract.phase == PathingWaypointPhase::RunAction)
            .expect("expected RunAction contract");
        assert_eq!(
            run_action_contract.pending_dependencies,
            vec![
                PathingMovementDependency::InputDispatch,
                PathingMovementDependency::ActionHandlers
            ]
        );
    }
}

#[test]
fn pathing_execution_plan_models_pick_around_action_sequence() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "pick route", "type": "collect", "map_name": "Teyvat" },
            "positions": [
                { "x": 10.0, "y": 20.0, "type": "path", "action": "pick_around", "action_params": "2" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();
    let waypoint = &plan.segments[0].waypoints[0];

    assert_eq!(
        waypoint.declared_action_use,
        Some(PathingActionUseWaypointType::Custom)
    );
    assert!(waypoint.phases.contains(&PathingWaypointPhase::RunAction));
    let Some(PathingActionPlan::PickAround(pick_around)) = &waypoint.action_plan else {
        panic!("expected pick_around action plan");
    };
    assert_eq!(pick_around.action_code, "pick_around");
    assert_eq!(pick_around.raw_params.as_deref(), Some("2"));
    assert_eq!(pick_around.turns, 2);
    assert_eq!(pick_around.turn_parse_error, None);
    assert_eq!(pick_around.speed, 1.1);
    assert_eq!(pick_around.circle_segments_per_turn, 6);
    assert_eq!(pick_around.circle_start_ms, 600);
    assert_eq!(pick_around.circle_interval_ms, 400);
    assert_eq!(pick_around.view_reset_base_ms, 350);
    assert!(pick_around.executor_ready);
    assert_eq!(pick_around.turn_plans.len(), 2);
    assert_eq!(pick_around.steps.len(), 68);

    let first_turn = &pick_around.turn_plans[0];
    assert_eq!(first_turn.turn_index, 0);
    assert_eq!(first_turn.edge_delay_ms, 545);
    assert_eq!(first_turn.move_backward_forward_ms, 200);
    assert_eq!(first_turn.move_left_forward_ms, 312);
    assert!((first_turn.radius_time_ms - 311.8097605735693).abs() < 0.000001);

    let second_turn = &pick_around.turn_plans[1];
    assert_eq!(second_turn.turn_index, 1);
    assert_eq!(second_turn.edge_delay_ms, 909);
    assert_eq!(second_turn.move_backward_forward_ms, 498);
    assert_eq!(second_turn.move_left_forward_ms, 424);

    assert_eq!(
        pick_around.steps[0],
        PathingPickAroundStep::Key {
            turn_index: 0,
            key: KeyId::MOUSE_MIDDLE_BUTTON,
            press: PathingInputPress::KeyPress
        }
    );
    assert_eq!(
        pick_around.steps[1],
        PathingPickAroundStep::Delay {
            turn_index: 0,
            milliseconds: 500
        }
    );
    assert_eq!(
        pick_around.steps[2],
        PathingPickAroundStep::GenshinAction {
            turn_index: 0,
            action: GenshinAction::MoveBackward,
            press: PathingInputPress::KeyPress
        }
    );
    assert!(pick_around.steps.iter().any(|step| {
        *step
            == PathingPickAroundStep::GenshinAction {
                turn_index: 0,
                action: GenshinAction::MoveLeft,
                press: PathingInputPress::KeyDown,
            }
    }));
    assert!(pick_around.steps.iter().any(|step| {
        *step
            == PathingPickAroundStep::Delay {
                turn_index: 1,
                milliseconds: 909,
            }
    }));

    let run_action_contract = plan.movement_contract.segments[0].waypoints[0]
        .phase_contracts
        .iter()
        .find(|contract| contract.phase == PathingWaypointPhase::RunAction)
        .expect("expected RunAction contract");
    assert_eq!(
        run_action_contract.pending_dependencies,
        vec![PathingMovementDependency::InputDispatch]
    );
}

#[test]
fn pathing_execution_plan_preserves_pick_around_turn_parse_fallbacks() {
    let task = PathingTask::from_json(
        r#"{
            "info": { "name": "pick fallback route", "type": "collect", "map_name": "Teyvat" },
            "positions": [
                { "x": 10.0, "y": 20.0, "type": "path", "action": "pick_around", "action_params": "bad" },
                { "x": 11.0, "y": 21.0, "type": "path", "action": "pick_around", "action_params": "0" },
                { "x": 12.0, "y": 22.0, "type": "path", "action": "pick_around", "action_params": "-2" }
            ]
        }"#,
    )
    .unwrap();

    let plan = task.execution_plan();

    let Some(PathingActionPlan::PickAround(invalid)) = &plan.segments[0].waypoints[0].action_plan
    else {
        panic!("expected invalid pick_around action plan");
    };
    assert_eq!(invalid.turns, 1);
    assert!(invalid.turn_parse_error.as_ref().unwrap().contains("bad"));
    assert_eq!(invalid.turn_plans.len(), 1);
    assert_eq!(invalid.steps.len(), 34);

    let Some(PathingActionPlan::PickAround(zero)) = &plan.segments[0].waypoints[1].action_plan
    else {
        panic!("expected zero-turn pick_around action plan");
    };
    assert_eq!(zero.turns, 0);
    assert!(zero.turn_plans.is_empty());
    assert!(zero.steps.is_empty());

    let Some(PathingActionPlan::PickAround(negative)) = &plan.segments[0].waypoints[2].action_plan
    else {
        panic!("expected negative-turn pick_around action plan");
    };
    assert_eq!(negative.turns, -2);
    assert!(negative.turn_plans.is_empty());
    assert!(negative.steps.is_empty());
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
