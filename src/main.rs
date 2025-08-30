mod hawkbit;
use hawkbit::HawkbitConfig;

use tracing_subscriber;

#[tokio::main]
async fn main() {
    let config = HawkbitConfig::from_env();
    let client = hawkbit::HawkbitMgmtClient::from_config(&config);

    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    // let targets = client.get_targets(None).await.unwrap();
    // let first_target = targets.first().unwrap();
    // println!("First target: {:?}\n\n", first_target);
    let controller_id = "meticulousAcidicWhippedTopping-000022".to_string();

    let target = client.get_target(&controller_id).await.unwrap();
    println!("Target: {:?}\n\n", target);

    let attributes = client
        .get_target_attributes(&controller_id, None)
        .await
        .unwrap();
    println!("Attributes: {:?}\n\n", attributes);

    let actions = client
        .get_target_actions(&controller_id, Some(5))
        .await
        .unwrap();
    println!("Actions: {:?}\n\n", actions);

    let first_action = actions.first().unwrap();
    println!("First action: {:?}\n\n", first_action);

    let action_details = client
        .get_action_detail(&controller_id, &first_action.id)
        .await
        .unwrap();
    println!("Action details: {:?}\n\n", action_details);

    let action_status = client
        .get_action_status(&controller_id, &first_action.id)
        .await
        .unwrap();
    for status in action_status {
        println!("{}: {:?}", status.event_type, status.messages);
    }
}
