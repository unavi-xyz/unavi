pub mod world {
    pub mod webrtc {
        pub mod common {
            include!(concat!(env!("OUT_DIR"), "/world.webrtc.common.rs"));
        }

        pub mod request {
            include!(concat!(env!("OUT_DIR"), "/world.webrtc.request.rs"));
        }

        pub mod response {
            include!(concat!(env!("OUT_DIR"), "/world.webrtc.response.rs"));
        }
    }

    pub mod websocket {
        pub mod request {
            include!(concat!(env!("OUT_DIR"), "/world.websocket.request.rs"));
        }

        pub mod response {
            include!(concat!(env!("OUT_DIR"), "/world.websocket.response.rs"));
        }

        pub mod signaling {
            pub mod common {
                include!(concat!(
                    env!("OUT_DIR"),
                    "/world.websocket.signaling.common.rs"
                ));
            }

            pub mod request {
                include!(concat!(
                    env!("OUT_DIR"),
                    "/world.websocket.signaling.request.rs"
                ));
            }

            pub mod response {
                include!(concat!(
                    env!("OUT_DIR"),
                    "/world.websocket.signaling.response.rs"
                ));
            }
        }
    }
}
