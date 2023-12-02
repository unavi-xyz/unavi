pub mod webrtc {
    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/webrtc.common.rs"));
    }

    pub mod request {
        include!(concat!(env!("OUT_DIR"), "/webrtc.request.rs"));
    }

    pub mod response {
        include!(concat!(env!("OUT_DIR"), "/webrtc.response.rs"));
    }
}

pub mod websocket {
    pub mod request {
        include!(concat!(env!("OUT_DIR"), "/websocket.request.rs"));
    }

    pub mod response {
        include!(concat!(env!("OUT_DIR"), "/websocket.response.rs"));
    }

    pub mod signaling {
        pub mod common {
            include!(concat!(env!("OUT_DIR"), "/websocket.signaling.common.rs"));
        }

        pub mod request {
            include!(concat!(env!("OUT_DIR"), "/websocket.signaling.request.rs"));
        }

        pub mod response {
            include!(concat!(env!("OUT_DIR"), "/websocket.signaling.response.rs"));
        }
    }
}
