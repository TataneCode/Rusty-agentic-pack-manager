pub mod domain {
    pub mod pack {
        pub mod entity;
        pub mod value_objects;
    }
}

pub mod application {
    pub mod pack {
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
        pub mod loader;
    }
    pub mod filesystem {
        pub mod explorer;
    }
}

pub mod presentation {
    pub mod tui {
        pub mod app;
        pub mod common;
        pub mod event;
        pub mod router;
        pub mod packs {
            pub mod actions;
            pub mod presenter;
            pub mod view;
        }
        pub mod explorer {
            pub mod actions;
            pub mod presenter;
            pub mod view;
        }
        pub mod viewer {
            pub mod actions;
            pub mod presenter;
            pub mod view;
        }
    }
}
