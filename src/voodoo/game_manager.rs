use std::sync::mpsc::{Sender, channel};
use std::thread;
use rustc_serialize::json;
use voodoo::Scene;

pub struct GameManager {
    current_scene: Scene,
    client_queue: Vec<Sender<String>>,
    num_clients: usize
}

impl GameManager {
    fn new() -> GameManager {
        GameManager {
            current_scene: Scene::new(),
            client_queue: Vec::new(),
            num_clients: 2
        }
    }
}

impl GameManager {
    pub fn run() -> Sender<VoodooMessage> {
        let (spawn_sender, spawn_receiver) = channel();
        let spawn_sender = spawn_sender.clone();
        thread::spawn(move || {
            let (event_sender, event_receiver) = channel();
            if let Ok(_) = spawn_sender.send(event_sender.clone()) {
                let mut manager = GameManager::new();
                while let Ok(message) = event_receiver.recv() {
                    match message {
                        VoodooMessage::TurnAction(action, response_channel) => {
                            manager.current_scene.head += action.head;
                            manager.current_scene.body += action.body;
                            manager.current_scene.arm_left += action.arm_left;
                            manager.current_scene.arm_right += action.arm_right;
                            manager.current_scene.leg_left += action.leg_left;
                            manager.current_scene.leg_right += action.leg_right;

                            manager.client_queue.push(response_channel);
                            if manager.client_queue.len() == manager.num_clients {
                                let response = VoodooResponse::new(&manager.current_scene);
                                let body_content = json::encode(&response).unwrap();
                                for client in &manager.client_queue {
                                    client.send(body_content.to_string()).unwrap();
                                }
                                manager.current_scene.next();
                                manager.client_queue.clear()
                            }
                        }
                    }
                }
            }
        });
        spawn_receiver.recv().unwrap()
    }
}

pub enum VoodooMessage {
    TurnAction(ActionContent, Sender<String>)
}

#[derive(RustcDecodable, RustcEncodable)]
struct VoodooResponse {
    next_level: isize,
    arm_left_score: isize,
    arm_right_score: isize,
    head_score: isize,
    leg_left_score: isize,
    leg_right_score: isize,
    body_score: isize,
    total_score: isize,
    current_level: isize
}

impl VoodooResponse {
    fn new(scene: &Scene) -> VoodooResponse {
        VoodooResponse {
            next_level: scene.scene_number+1,
            arm_left_score: scene.arm_left,
            arm_right_score: scene.arm_right,
            head_score: scene.head,
            leg_left_score: scene.leg_left,
            leg_right_score: scene.leg_right,
            body_score: scene.body,
            total_score: scene.arm_left + scene.arm_right + scene.head + scene.leg_left + scene.leg_right + scene.body,
            current_level: scene.scene_number
        }
    }
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct ActionContent {
    head: isize,
    body: isize,
    arm_left: isize,
    arm_right: isize,
    leg_left: isize,
    leg_right: isize
}