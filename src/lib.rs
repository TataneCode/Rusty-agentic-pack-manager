pub mod domain {
    pub mod pack {
        pub mod entity;
        pub mod error;
        pub mod installed;
        pub mod value_objects;
    }
}

pub mod application {
    pub mod error;
    pub mod pack {
        pub mod dto;
        pub mod service;
        pub mod traits;
    }
    pub mod pack_create {
        pub mod dto;
        pub mod service;
        pub mod traits;
    }
    pub mod pack_view {
        pub mod dto;
        pub mod service;
        pub mod traits;
    }
    pub mod explorer {
        pub mod dto;
        pub mod service;
        pub mod traits;
    }
}

pub mod infrastructure {
    pub mod pack {
        pub mod content;
        pub mod index;
        pub mod loader;
        pub mod writer;
    }
    pub mod filesystem {
        pub mod explorer;
    }
}

pub mod presentation {
    pub mod cli;
    pub mod library;
    pub mod tui {
        pub mod app;
        pub mod common;
        pub mod event;
        pub mod router;
        pub mod split_view;
        pub mod viewer;
    }
}
