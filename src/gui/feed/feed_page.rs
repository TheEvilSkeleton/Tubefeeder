use gdk::subclass::prelude::ObjectSubclassIsExt;
use tf_join::{AnyVideo, Joiner};
use tf_playlist::PlaylistManager;

gtk::glib::wrapper! {
    pub struct FeedPage(ObjectSubclass<imp::FeedPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl FeedPage {
    pub fn setup(&self, playlist_manager: PlaylistManager<String, AnyVideo>, joiner: Joiner) {
        self.imp().playlist_manager.replace(Some(playlist_manager));
        self.imp().joiner.replace(Some(joiner));
        self.imp().setup(&self);
    }
}

pub mod imp {
    use std::cell::Cell;
    use std::cell::RefCell;

    use gdk::glib::clone;
    use gdk::glib::MainContext;
    use gdk::glib::ParamFlags;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecBoolean;
    use gdk::glib::PRIORITY_DEFAULT;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use tf_core::ErrorStore;
    use tf_core::Generator;
    use tf_join::AnyVideo;
    use tf_join::Joiner;
    use tf_playlist::PlaylistManager;

    use crate::gui::feed::error_label::ErrorLabel;
    use crate::gui::feed::feed_item_object::VideoObject;
    use crate::gui::feed::feed_list::FeedList;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/feed_page.ui")]
    pub struct FeedPage {
        #[template_child]
        pub(super) feed_list: TemplateChild<FeedList>,

        #[template_child]
        pub(super) btn_reload: TemplateChild<gtk::Button>,

        #[template_child]
        pub(super) error_label: TemplateChild<ErrorLabel>,

        reloading: Cell<bool>,

        pub(super) playlist_manager: RefCell<Option<PlaylistManager<String, AnyVideo>>>,
        pub(super) joiner: RefCell<Option<Joiner>>,
        error_store: RefCell<ErrorStore>,
    }

    impl FeedPage {
        fn setup_reload(&self, obj: &super::FeedPage) {
            let joiner = self
                .joiner
                .borrow()
                .clone()
                .expect("Joiner should be set up");

            let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
            let sender = sender.clone();
            let joiner = joiner.clone();
            let error_store = self.error_store.borrow().clone();

            self.btn_reload.connect_clicked(
                clone!(@strong obj as s, @strong joiner, @strong error_store => move |_| {
                    log::debug!("Reloading");
                    s.set_property("reloading", &true);

                    let sender = sender.clone();
                    let joiner = joiner.clone();
                    let error_store = error_store.clone();
                    tokio::spawn(async move {
                        let videos = joiner.generate(&error_store).await;
                        let _ = sender.send(videos);
                    });
                }),
            );
            receiver.attach(
                None,
                clone!(@strong obj as s => @default-return Continue(false), move |videos| {
                    let video_objects = videos.into_iter().map(VideoObject::new).collect::<Vec<_>>();
                    s.imp().feed_list.get().set_items(video_objects);
                    s.set_property("reloading", &false);
                    Continue(true)
                }),
            );

            // Setup Error Label
            self.error_label
                .set_error_store(self.error_store.borrow().clone());

            // Simulate reload on startup.
            self.btn_reload.emit_clicked();

            self.joiner.replace(Some(joiner));
        }

        pub(super) fn setup(&self, obj: &super::FeedPage) {
            self.feed_list.set_playlist_manager(
                self.playlist_manager
                    .borrow()
                    .clone()
                    .expect("PlaylistManager has to be set up"),
            );
            self.setup_reload(obj);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FeedPage {
        const NAME: &'static str = "TFFeedPage";
        type Type = super::FeedPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FeedPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecBoolean::new(
                    "reloading",
                    "reloading",
                    "reloading",
                    false,
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "reloading" => {
                    let _ = self.reloading.replace(
                        value
                            .get()
                            .expect("The property 'reloading' of TFWindow has to be boolean"),
                    );
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "reloading" => self.reloading.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for FeedPage {}
    impl BoxImpl for FeedPage {}
}
