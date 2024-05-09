// SPDX-FileCopyrightText: © 2024 David Bliss
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::prelude::OrientableExt;

use itertools::Itertools;
use relm4::gtk;
use relm4::gtk::gdk;
use relm4::gtk::gdk_pixbuf;
use relm4::gtk::prelude::WidgetExt;
use relm4::typed_view::grid::{RelmGridItem, TypedGridView};
use relm4::*;
use std::path;
use std::sync::Arc;

use crate::app::SharedState;
use crate::app::ActiveView;
use crate::app::ViewName;

use tracing::{event, Level};

#[derive(Debug)]
struct PhotoGridItem {
    folder_name: String,

    // Folder album cover
    picture: Arc<fotema_core::visual::Visual>,
}

struct Widgets {
    picture: gtk::Picture,
    label: gtk::Label,
}
#[derive(Debug)]
pub enum FolderPhotosInput {
    Activate,

    // Reload photos from database
    Refresh,

    FolderSelected(u32), // Index into photo grid vector
}

#[derive(Debug)]
pub enum FolderPhotosOutput {
    FolderSelected(path::PathBuf),
}

impl RelmGridItem for PhotoGridItem {
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn setup(_item: &gtk::ListItem) -> (gtk::Box, Widgets) {
        relm4::view! {
           my_box = gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::Clamp {
                    set_maximum_size: 200,
                    set_orientation: gtk::Orientation::Horizontal,

                    gtk::Frame {
                        #[name(picture)]
                        gtk::Picture {
                            set_can_shrink: true,
                            set_valign: gtk::Align::Center,
                            set_width_request: 200,
                            set_height_request: 200,
                        }
                    }
                },

                #[name(label)]
                gtk::Label {
                    add_css_class: "caption-heading",
                    set_margin_top: 4,
                    set_margin_bottom: 12,
                },
            }
        }

        let widgets = Widgets { picture, label };

        (my_box, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets
            .label
            .set_text(format!("{}", self.folder_name).as_str());

        if self.picture.thumbnail_path.as_ref().is_some_and(|x| x.exists())
        {
            widgets
                .picture
                .set_filename(self.picture.thumbnail_path.clone());
        } else {
            let pb = gdk_pixbuf::Pixbuf::from_resource_at_scale(
                "/app/fotema/Fotema/icons/scalable/actions/image-missing-symbolic.svg",
                200, 200, true
            ).unwrap();
            let img = gdk::Texture::for_pixbuf(&pb);
            widgets.picture.set_paintable(Some(&img));
        }
    }

    fn unbind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.picture.set_filename(None::<&path::Path>);
    }
}

pub struct FolderPhotos {
    state: SharedState,
    active_view: ActiveView,
    photo_grid: TypedGridView<PhotoGridItem, gtk::SingleSelection>,
}

#[relm4::component(pub)]
impl SimpleComponent for FolderPhotos {
    type Init = (SharedState, ActiveView);
    type Input = FolderPhotosInput;
    type Output = FolderPhotosOutput;

    view! {
        gtk::ScrolledWindow {
            //set_propagate_natural_height: true,
            //set_has_frame: true,
            set_vexpand: true,

            #[local_ref]
            pictures_box -> gtk::GridView {
                set_orientation: gtk::Orientation::Vertical,
                set_single_click_activate: true,
                //set_max_columns: 3,

                connect_activate[sender] => move |_, idx| {
                    sender.input(FolderPhotosInput::FolderSelected(idx))
                }
            }
        }
    }

    fn init(
        (state, active_view): Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let photo_grid = TypedGridView::new();

        let model = FolderPhotos { state, active_view, photo_grid };

        let pictures_box = &model.photo_grid.view;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            FolderPhotosInput::Activate => {
                *self.active_view.write() = ViewName::Folders;
                if self.photo_grid.is_empty() {
                    self.refresh();
                }
            }
            FolderPhotosInput::Refresh => {
                if *self.active_view.read() == ViewName::Folders {
                    self.refresh();
                } else {
                    self.photo_grid.clear();
                }
            }
            FolderPhotosInput::FolderSelected(index) => {
                event!(Level::DEBUG, "Folder selected index: {}", index);
                if let Some(item) = self.photo_grid.get_visible(index) {
                    let item = item.borrow();
                    event!(Level::DEBUG, "Folder selected item: {}", item.folder_name);

                    let _ = sender
                        .output(FolderPhotosOutput::FolderSelected(item.picture.parent_path.clone()));
                }
            }
        }
    }
}

impl FolderPhotos {
    fn refresh(&mut self) {
        let all = {
            let data = self.state.read();
            data.clone()
                .into_iter()
               // .filter(|x| x.thumbnail_path.exists())
                .sorted_by_key(|pic| pic.parent_path.clone())
                .group_by(|pic| pic.parent_path.clone())
                //.collect::<Vec<Arc<Visual>>>()
        };

        let mut pictures = Vec::new();

        for (_key, mut group) in &all {
            let first = group.nth(0).expect("Groups can't be empty");
            let album = PhotoGridItem {
                folder_name: first.folder_name().unwrap_or("-".to_string()),
                picture: first.clone(),
            };
            pictures.push(album);
        }

        pictures.sort_by_key(|pic| pic.folder_name.clone());

        self.photo_grid.clear();
        self.photo_grid.extend_from_iter(pictures.into_iter());

        // NOTE folder view is not sorted by a timestamp, so don't scroll to end.
    }
}
