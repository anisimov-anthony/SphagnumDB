// SphagnumDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use libp2p::Multiaddr;
use sphagnumdb::core::{
    commands::{generic::GenericCommand, string::StringCommand, Command, CommandResult},
    sphagnum::SphagnumNode,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

#[tokio::test]
async fn test_config_cluster_and_check_replication() {
    // Arrange
    let mut sp1 = SphagnumNode::new().unwrap();
    let mut sp2 = SphagnumNode::new().unwrap();
    let mut sp3 = SphagnumNode::new().unwrap();

    sp1.listen_on("/ip4/127.0.0.1/tcp/3301".parse::<Multiaddr>().unwrap())
        .unwrap();
    sp2.listen_on("/ip4/127.0.0.1/tcp/3302".parse::<Multiaddr>().unwrap())
        .unwrap();
    sp3.listen_on("/ip4/127.0.0.1/tcp/3303".parse::<Multiaddr>().unwrap())
        .unwrap();

    sp1.dial("/ip4/127.0.0.1/tcp/3302").unwrap();
    sp1.dial("/ip4/127.0.0.1/tcp/3303").unwrap();
    sp2.dial("/ip4/127.0.0.1/tcp/3301").unwrap();
    sp2.dial("/ip4/127.0.0.1/tcp/3303").unwrap();
    sp3.dial("/ip4/127.0.0.1/tcp/3301").unwrap();
    sp3.dial("/ip4/127.0.0.1/tcp/3302").unwrap();

    let peer_id1 = sp1.peer_id().unwrap();
    let peer_id2 = sp2.peer_id().unwrap();
    let peer_id3 = sp3.peer_id().unwrap();
    sp1.add_to_replica_set(peer_id2).unwrap();
    sp1.add_to_replica_set(peer_id3).unwrap();
    sp2.add_to_replica_set(peer_id1).unwrap();
    sp2.add_to_replica_set(peer_id3).unwrap();
    sp3.add_to_replica_set(peer_id1).unwrap();
    sp3.add_to_replica_set(peer_id2).unwrap();

    let sp_arc_1 = Arc::new(Mutex::new(sp1));
    let sp_arc_2 = Arc::new(Mutex::new(sp2));
    let sp_arc_3 = Arc::new(Mutex::new(sp3));

    let handle_events_1 = {
        let sp_arc_1 = Arc::clone(&sp_arc_1);
        tokio::spawn(async move {
            loop {
                let mut sphagnum = sp_arc_1.lock().await;
                if let Err(e) = sphagnum.handle_event().await {
                    eprintln!("Error handling event on sp1: {}", e);
                }
            }
        })
    };

    let handle_events_2 = {
        let sp_arc_2 = Arc::clone(&sp_arc_2);
        tokio::spawn(async move {
            loop {
                let mut sphagnum = sp_arc_2.lock().await;
                if let Err(e) = sphagnum.handle_event().await {
                    eprintln!("Error handling event on sp2: {}", e);
                }
            }
        })
    };

    let handle_events_3 = {
        let sp_arc_3 = Arc::clone(&sp_arc_3);
        tokio::spawn(async move {
            loop {
                let mut sphagnum = sp_arc_3.lock().await;
                if let Err(e) = sphagnum.handle_event().await {
                    eprintln!("Error handling event on sp3: {}", e);
                }
            }
        })
    };

    sleep(Duration::from_millis(1000)).await;

    //let mut node1 = sp_arc_1.lock().await;
    //let mut node2 = sp_arc_2.lock().await;
    //let mut node3 = sp_arc_3.lock().await;

    // Act & Assert

    // Step 1: assert Set command
    let set_command = Command::String(StringCommand::Set {
        key: "key".to_string(),
        value: "value".to_string(),
    });
    let get_command = Command::String(StringCommand::Get {
        key: "key".to_string(),
    });

    // sp1 -> request -> sp2
    {
        let mut node1 = sp_arc_1.lock().await;
        let result = node1.send_request_to_sphagnum(peer_id2, set_command).await;
        assert!(
            result.is_ok(),
            "Failed to send Set request from sp1 to sp2: {:?}",
            result
        );
    }

    // Time for replication
    sleep(Duration::from_millis(5000)).await;

    let get_sp1 = {
        let mut node1 = sp_arc_1.lock().await;
        node1.handle_command(get_command.clone()).unwrap()
    };
    let get_sp2 = {
        let mut node2 = sp_arc_2.lock().await;
        node2.handle_command(get_command.clone()).unwrap()
    };
    let get_sp3 = {
        let mut node3 = sp_arc_3.lock().await;
        node3.handle_command(get_command.clone()).unwrap()
    };
    assert_eq!(get_sp1, CommandResult::String("value".to_string()));
    assert_eq!(get_sp2, CommandResult::String("value".to_string()));
    assert_eq!(get_sp3, CommandResult::String("value".to_string()));

    // Step 2: assert Append command
    let append_command = Command::String(StringCommand::Append {
        key: "key".to_string(),
        value: "appended_part".to_string(),
    });
    let get_command = Command::String(StringCommand::Get {
        key: "key".to_string(),
    });

    // sp2 -> request -> sp1
    {
        let mut node2 = sp_arc_2.lock().await;
        let result = node2
            .send_request_to_sphagnum(peer_id1, append_command)
            .await;
        assert!(
            result.is_ok(),
            "Failed to send Append request from sp2 to sp1: {:?}",
            result
        );
    }

    // Time for replication
    sleep(Duration::from_millis(5000)).await;

    let get_sp1 = {
        let mut node1 = sp_arc_1.lock().await;
        node1.handle_command(get_command.clone()).unwrap()
    };
    let get_sp2 = {
        let mut node2 = sp_arc_2.lock().await;
        node2.handle_command(get_command.clone()).unwrap()
    };
    let get_sp3 = {
        let mut node3 = sp_arc_3.lock().await;
        node3.handle_command(get_command.clone()).unwrap()
    };
    assert_eq!(
        get_sp1,
        CommandResult::String("valueappended_part".to_string())
    );
    assert_eq!(
        get_sp2,
        CommandResult::String("valueappended_part".to_string())
    );
    assert_eq!(
        get_sp3,
        CommandResult::String("valueappended_part".to_string())
    );

    // Step 3: assert Delete command
    let delete_command = Command::Generic(GenericCommand::Delete {
        keys: vec!["key".to_string()],
    });
    let get_command = Command::String(StringCommand::Get {
        key: "key".to_string(),
    });

    // sp2 -> request -> sp3
    {
        let mut node2 = sp_arc_2.lock().await;
        let result = node2
            .send_request_to_sphagnum(peer_id3, delete_command)
            .await;
        assert!(
            result.is_ok(),
            "Failed to send Delete request from sp2 to sp3: {:?}",
            result
        );
    }

    // Time for replication
    sleep(Duration::from_millis(5000)).await;

    let get_sp1 = {
        let mut node1 = sp_arc_1.lock().await;
        node1.handle_command(get_command.clone()).unwrap()
    };
    let get_sp2 = {
        let mut node2 = sp_arc_2.lock().await;
        node2.handle_command(get_command.clone()).unwrap()
    };
    let get_sp3 = {
        let mut node3 = sp_arc_3.lock().await;
        node3.handle_command(get_command.clone()).unwrap()
    };
    assert_eq!(get_sp1, CommandResult::Nil);
    assert_eq!(get_sp2, CommandResult::Nil);
    assert_eq!(get_sp3, CommandResult::Nil);

    // Step 4: assert Exists command
    let exists_command = Command::Generic(GenericCommand::Exists {
        keys: vec!["key".to_string()],
    });

    let exists_sp1 = {
        let mut node1 = sp_arc_1.lock().await;
        node1.handle_command(exists_command.clone()).unwrap()
    };
    let exists_sp2 = {
        let mut node2 = sp_arc_2.lock().await;
        node2.handle_command(exists_command.clone()).unwrap()
    };
    let exists_sp3 = {
        let mut node3 = sp_arc_3.lock().await;
        node3.handle_command(exists_command.clone()).unwrap()
    };
    assert_eq!(exists_sp1, CommandResult::Int(0));
    assert_eq!(exists_sp2, CommandResult::Int(0));
    assert_eq!(exists_sp3, CommandResult::Int(0));

    handle_events_1.abort();
    handle_events_2.abort();
    handle_events_3.abort();
}
