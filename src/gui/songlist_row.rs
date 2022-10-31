//
// songlist_row.rs
// Copyright (C) 2022 gmg137 <gmg137 AT live.com>
// Distributed under terms of the GPL-3.0-or-later license.
//
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate, *};

use crate::application::Action;
use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, SendWeakRef, Sender, Value};
use ncm_api::{SongInfo, SongList};
use once_cell::sync::{Lazy, OnceCell};
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};

glib::wrapper! {
    pub struct SonglistRow(ObjectSubclass<imp::SonglistRow>)
        @extends Widget, ListBoxRow,
        @implements Accessible, Actionable, Buildable, ConstraintTarget;
}

impl Default for SonglistRow {
    fn default() -> Self {
        Self::new()
    }
}

impl SonglistRow {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn set_sender(&self, sender: Sender<Action>) {
        self.imp().sender.set(sender).unwrap();
    }

    pub fn set_from_song_info(&self, si: &SongInfo) {
        self.imp().song_id.set(si.id).unwrap();
        self.imp().album_id.set(si.album_id).unwrap();
        self.imp().cover_url.set(si.pic_url.to_owned()).unwrap();

        self.set_tooltip_text(Some(&si.name));
        self.set_name(&si.name);
        self.set_singer(&si.singer);
        self.set_album(&si.album);
        self.set_duration(&si.duration);
    }

    pub fn switch_image(&self, visible: bool) {
        let imp = self.imp();
        imp.play_icon.set_visible(visible);
    }

    fn set_name(&self, label: &str) {
        let imp = self.imp();
        imp.title_label.set_label(label);
    }

    fn set_singer(&self, label: &str) {
        let imp = self.imp();
        imp.artist_label.set_label(label);
    }

    fn set_album(&self, label: &str) {
        let imp = self.imp();
        imp.album_label.set_label(label);
    }

    fn set_duration(&self, label: &str) {
        let imp = self.imp();
        imp.duration_label.set_label(label);
    }
}

#[gtk::template_callbacks]
impl SonglistRow {
    #[template_callback]
    fn on_click(&self) {
        self.emit_activate();
    }
}

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/gitee/gmg137/NeteaseCloudMusicGtk4/gtk/songlist-row.ui")]
    pub struct SonglistRow {
        #[template_child]
        pub play_icon: TemplateChild<Image>,
        #[template_child]
        pub title_label: TemplateChild<Label>,
        #[template_child]
        pub artist_label: TemplateChild<Label>,
        #[template_child]
        pub album_label: TemplateChild<Label>,
        #[template_child]
        pub duration_label: TemplateChild<Label>,
        #[template_child]
        pub like_button: TemplateChild<Button>,
        #[template_child]
        pub album_button: TemplateChild<Button>,

        pub sender: OnceCell<Sender<Action>>,
        pub song_id: OnceCell<u64>,
        pub album_id: OnceCell<u64>,
        pub cover_url: OnceCell<String>,
        like: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SonglistRow {
        const NAME: &'static str = "SonglistRow";
        type Type = super::SonglistRow;
        type ParentType = ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.bind_template_callbacks();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl SonglistRow {
        #[template_callback]
        fn like_button_clicked_cb(&self) {
            let sender = self.sender.get().unwrap();
            let s_send = SendWeakRef::from(self.obj().downgrade());
            let like = self.like.get();
            sender
                .send(Action::LikeSong(
                    self.song_id.get().unwrap().to_owned(),
                    !like,
                    Some(Arc::new(move |_| {
                        if let Some(s) = s_send.upgrade() {
                            s.set_property("like", !like);
                        }
                    })),
                ))
                .unwrap();
        }

        #[template_callback]
        fn album_button_clicked_cb(&self) {
            let sender = self.sender.get().unwrap();
            let songlist = SongList {
                id: self.album_id.get().unwrap().to_owned(),
                name: self.album_label.label().to_string(),
                cover_img_url: self.cover_url.get().unwrap().to_owned(),
            };
            sender
                .send(Action::ToAlbumPage(songlist.to_owned()))
                .unwrap();
        }
    }

    impl ObjectImpl for SonglistRow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.bind_property("like", &self.like_button.get(), "icon_name")
                .transform_to(|_, v: bool| {
                    Some(
                        (if v {
                            "starred-symbolic"
                        } else {
                            "non-starred-symbolic"
                        })
                        .to_string(),
                    )
                })
                .build();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecBoolean::builder("like").readwrite().build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "like" => {
                    let like = value.get().expect("The value needs to be of type `bool`.");
                    self.like.replace(like);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "like" => self.like.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for SonglistRow {}
    impl ListBoxRowImpl for SonglistRow {}
}
