mod hawkbit;
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use hawkbit::HawkbitConfig;

use tracing_subscriber;

use crate::hawkbit::DistributionSet;

// #[tokio::main]
// async fn main() {
//     // tracing_subscriber::fmt()
//     //     .with_max_level(tracing::Level::DEBUG)
//     //     .init();
//     let config = HawkbitConfig::from_env();
//     let client = hawkbit::HawkbitMgmtClient::from_config(&config);

//     let targets = client
//         .get_targets(Some("attribute.update_channel != attribute.build_channel"))
//         .await
//         .unwrap();
//     println!(
//         "Targets with mismatched update/build channels: {:?}",
//         targets.len()
//     );
//     for target in &targets {
//         let attributes = client
//             .get_target_attributes(&target.controller_id, None)
//             .await
//             .unwrap();
//         if attributes.get("update_channel").unwrap_or(&"".to_owned())
//             != attributes.get("build_channel").unwrap_or(&"".to_owned())
//         {
//             println!(
//                 "Target: {:?} {:?} v {:?}",
//                 target.controller_id,
//                 attributes.get("update_channel").unwrap_or(&"".to_owned()),
//                 attributes.get("build_channel").unwrap_or(&"".to_owned())
//             );
//         }
//     }
// }

#[tokio::main]
async fn main() {
    let config = HawkbitConfig::from_env();
    let client = hawkbit::HawkbitMgmtClient::from_config(&config);
    let mut dist_sets_lookup: HashMap<String, DistributionSet> = HashMap::new();
    let sets = client.get_distribution_sets(None).await.unwrap();
    for set in sets {
        let name = set.name.clone();
        if dist_sets_lookup.contains_key(&name) {
            if dist_sets_lookup.get(&name).unwrap().created_at >= set.created_at {
                // println!(
                //     "Skipping older distribution set: {:?} {:?}",
                //     name, set.created_at
                // );
                continue;
            } else {
                println!(
                    "Replacing distribution set with newer one: {:?} {:?}",
                    name, set.created_at
                );
            }
        } else {
            println!("Distribution set: {:?} {:?}", name, set.created_at);
        }
        dist_sets_lookup.insert(name, set);
    }
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    let targets = client.get_targets(Some("updatestatus == \"error\"")).await.unwrap();

    // let controller_id =   "meticulousAcidicWhippedTopping-000022".to_string();

    // let target = client.get_target(&controller_id).await.unwrap();
    // println!("Target: {:?}\n\n", target);
    let mut factory_machines = Vec::new();
    let mut canceled_actions: Vec<hawkbit::Action> = Vec::new();
    let mut status_map: HashMap<&String, Vec<&String>> = HashMap::new();
    let mut last_seen_map = HashMap::new();

    for target in &targets {
        // println!("Target: {:?}", target.controller_id);
        // if (target.controller_id != "meticulousDarkPumpkinSpiceLatteREL21Q-000021") {
        //     continue;
        // }
        println!("Target: {:?}", target.controller_id);
        client.target_request_attributes(target.controller_id.as_str()).await.unwrap();

        let is_sn_unset = target.controller_id.contains("-999");
        if is_sn_unset {
            factory_machines.push(target.controller_id.clone());
        }
        if let Some(last_seen) = target.last_controller_request_at {
            let now_ts = Utc::now().timestamp();
            let last_ts = last_seen / 1000;
            let bucket = if last_ts >= now_ts - 24 * 3600 {
                "today"
            } else if last_ts >= now_ts - 3 * 24 * 3600 {
                "last_3_days"
            } else if last_ts >= now_ts - 7 * 24 * 3600 {
                "last_week"
            } else if last_ts >= now_ts - 14 * 24 * 3600 {
                "last_2_weeks"
            } else if last_ts >= now_ts - 30 * 24 * 3600 {
                "last_month"
            } else if last_ts >= now_ts - 90 * 24 * 3600 {
                "last_3_months"
            } else if last_ts >= now_ts - 180 * 24 * 3600 {
                "last_6_months"
            } else {
                "longer"
            };
            let entry = last_seen_map.entry(bucket).or_insert(Vec::new());
            entry.push(&target.controller_id);
        }

        // println!("Full target: {:?}", target);
        let controller_id = &target.controller_id;
        let status = &target.update_status;

        // let actions = client
        //     .get_target_actions(&controller_id, Some(5), None)
        //     .await
        //     .unwrap();
        // println!("Actions: {:?}\n\n", actions);

        // let first_action = actions.first().unwrap();
        // println!("First action: {:?}\n\n", first_action);

        // for action in &actions {
        //     let action_details = client
        //         .get_action_detail(&controller_id, &action.id)
        //         .await
        //         .unwrap();
        //     println!("Action details: {:?}\n\n", action_details);
        // }

        // let action_status = client
        //     .get_action_status(&controller_id, &first_action.id)
        //     .await
        //     .unwrap();
        // for status in action_status {
        //     println!("{}: {:?}", status.event_type, status.messages);
        // }
        if let Some(s) = status {
            let entry = status_map.entry(s).or_insert(Vec::new());
            entry.push(&target.controller_id);
            // let attributes = client
            //     .get_target_attributes(&controller_id, None)
            //     .await
            //     .unwrap();
            // println!("Attributes: {:?}\n\n", attributes);

            if s == "error" {
                println!("Error target: {:?}", target);
                let attributes = client
                    .get_target_attributes(&controller_id, None)
                    .await
                    .unwrap();
                println!("Attributes: {:?}\n\n", attributes);
                let update_channel = attributes.get("update_channel").unwrap();
                let last_action: Vec<hawkbit::Action> = client
                    .get_target_actions(&controller_id, Some(1), None)
                    .await
                    .unwrap();
                // let action_details = client
                //     .get_action_detail(&controller_id, &last_action[0].id)
                //     .await
                //     .unwrap();
                // println!("Last action details: {:?}\n", action_details);
                let action_status = client
                    .get_action_status(&controller_id, &last_action[0].id)
                    .await
                    .unwrap();
                for status in action_status {
                    println!("{}: {:?}", status.event_type, status.messages);
                }
                println!("Update channel: {:?}", update_channel);
                let dist_set_name = update_channel.to_owned() + " EMMC";
                if !dist_sets_lookup.contains_key(&dist_set_name) {
                    println!(
                        "No distribution set found for update channel: {:?}",
                        update_channel
                    );
                    continue;
                } else {
                    println!(
                        "Found distribution set for update channel: {:?}",
                        dist_set_name
                    );
                }

                let dist_set = dist_sets_lookup.get(&dist_set_name).unwrap();

                println!(
                    "{:?}: Reassigning distribution set ID: {:?} / Name: {}",
                    controller_id, dist_set.id, dist_set.name
                );

                client
                    .assign_distribution(&controller_id, &dist_set.id)
                    .await
                    .unwrap();
                println!("Reassigned distribution set");
            }
        }

        let actions: Vec<hawkbit::Action> = client
            .get_target_actions(&controller_id, Some(50), Some("status==\"pending\""))
            .await
            .unwrap();
        let mut has_active = false;
        for action in &actions {
            if action.action_type != "update" {
                continue;
            }
            if !has_active && (action.status == "pending" || action.status == "updating") {
                has_active = true;
                continue;
            }
            if action.status != "finished" && action.detail_status != "running" && has_active {
                println!("Should cancel action: {:?}", action);
                canceled_actions.push(action.clone());
                client
                    .cancel_action(&controller_id, &action.id, false)
                    .await
                    .unwrap();
            }
        }
    }
    println!("\n\nStatus summary:");
    for (status, controllers) in &status_map {
        println!("{}:{:?}", status, controllers.len());
    }
    println!("Error targets: {:?}", &status_map.get(&"error".to_string()));
    println!("Canceled actions: {:?}", canceled_actions.len());

    for (last_seen, controllers) in &last_seen_map {
        println!(
            "Status: {:?}, Controllers: {:?}",
            last_seen,
            controllers.len()
        );
    }
    // println!(
    //     "Factory machines that we are going to delete: {:?}",
    //     factory_machines.len()
    // );
    // for target_to_delete in &factory_machines {
    //     println!("Target to delete: {:?}", target_to_delete);
    //     client.delete_target(target_to_delete).await.unwrap();
    // }
    let targets = client.get_targets(None).await.unwrap();
    println!("Remaining targets after deletion: {:?}", targets.len());
}
