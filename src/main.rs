mod hawkbit;
use std::collections::HashMap;

use hawkbit::HawkbitConfig;

use tracing_subscriber;

#[tokio::main]
async fn main() {
    let config = HawkbitConfig::from_env();
    let client = hawkbit::HawkbitMgmtClient::from_config(&config);

    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    let targets = client.get_targets(None).await.unwrap();
    let first_target = targets.first().unwrap();
    println!("First target: {:?}\n\n", first_target);
    // let controller_id = "meticulousAcidicWhippedTopping-000022".to_string();

    // let target = client.get_target(&controller_id).await.unwrap();
    // println!("Target: {:?}\n\n", target);

    // let attributes = client
    //     .get_target_attributes(&controller_id, None)
    //     .await
    //     .unwrap();
    // println!("Attributes: {:?}\n\n", attributes);

    let mut status_map: HashMap<&String, Vec<&String>> = HashMap::new();
    for target in &targets {
        println!("Target: {:?}", target.controller_id);

        let controller_id = &target.controller_id;
        let status = &target.update_status;
        if let Some(s) = status {
            let entry = status_map.entry(s).or_insert(Vec::new());
            entry.push(&target.controller_id);

            if s == "error" {
                println!("Error target: {:?}", target);
                let last_action: Vec<hawkbit::Action> = client
                    .get_target_actions(&controller_id, Some(1), None)
                    .await
                    .unwrap();
                let action_details = client
                    .get_action_detail(&controller_id, &last_action[0].id)
                    .await
                    .unwrap();
                println!("Last action details: {:?}\n", action_details);
                if let Some(distribution_set) = action_details.links.distribution_set {
                    println!("Distribution set: {:?}", distribution_set);
                    let id = distribution_set
                        .href
                        .unwrap()
                        .rsplit('/')
                        .next()
                        .unwrap()
                        .to_string()
                        .parse::<i64>()
                        .unwrap();
                    println!("Distribution set ID: {:?}", id);
                    client
                        .assign_distribution(&controller_id, &id)
                        .await
                        .unwrap();
                    println!("Reassigned distribution set");
                }
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
            if action.status != "finished" && has_active {
                println!("Should cancel action: {:?}", action);
                client
                    .cancel_action(&controller_id, &action.id, false)
                    .await
                    .unwrap();
            }
        }
    }
    for (status, controllers) in &status_map {
        println!(
            "Status: {:?}, Controllers: {:?}\n",
            status,
            controllers.len()
        );
    }
    println!(
        "Error targets: {:?}\n\n",
        &status_map.get(&"error".to_string())
    );

    // let actions = client
    //     .get_target_actions(&controller_id, Some(5))
    //     .await
    //     .unwrap();
    // println!("Actions: {:?}\n\n", actions);

    // let first_action = actions.first().unwrap();
    // println!("First action: {:?}\n\n", first_action);

    // let action_details = client
    //     .get_action_detail(&controller_id, &first_action.id)
    //     .await
    //     .unwrap();
    // println!("Action details: {:?}\n\n", action_details);

    // let action_status = client
    //     .get_action_status(&controller_id, &first_action.id)
    //     .await
    //     .unwrap();
    // for status in action_status {
    //     println!("{}: {:?}", status.event_type, status.messages);
    // }
}
