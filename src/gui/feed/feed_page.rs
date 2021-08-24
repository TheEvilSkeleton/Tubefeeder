/*
 * Copyright 2021 Julian Schmidhuber <github@schmiddi.anonaddy.com>
 *
 * This file is part of Tubefeeder.
 *
 * Tubefeeder is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Tubefeeder is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Tubefeeder.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::gui::app::AppMsg;
use crate::gui::feed::feed_item::{FeedListItem, FeedListItemMsg};
use crate::gui::lazy_list::{LazyList, LazyListMsg, ListElementBuilder};
// use crate::youtube_feed::{Entry, Feed};

use tf_join::AnyVideo;

use gtk::prelude::*;
use gtk::Orientation::Vertical;
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

pub struct FeedElementBuilder {
    chunks: Vec<Vec<(AnyVideo, StreamHandle<AppMsg>)>>,
}

impl FeedElementBuilder {
    fn new(feed: Box<dyn Iterator<Item = AnyVideo>>, app_stream: StreamHandle<AppMsg>) -> Self {
        FeedElementBuilder {
            chunks: feed
                .map(|v| (v.clone(), app_stream.clone()))
                .collect::<Vec<(AnyVideo, StreamHandle<AppMsg>)>>()
                .chunks(10)
                .map(|slice| Vec::from(slice))
                .collect::<Vec<Vec<(AnyVideo, StreamHandle<AppMsg>)>>>(),
        }
    }
}

impl ListElementBuilder<FeedListItem> for FeedElementBuilder {
    fn poll(&mut self) -> Vec<(AnyVideo, StreamHandle<AppMsg>)> {
        if !self.chunks.is_empty() {
            self.chunks.remove(0)
        } else {
            vec![]
        }
    }

    fn add_stream(&mut self, stream: StreamHandle<FeedListItemMsg>) {
        stream.emit(FeedListItemMsg::SetImage);
    }

    fn get_clicked_signal(&self) -> Option<FeedListItemMsg> {
        Some(FeedListItemMsg::Clicked)
    }
}

#[derive(Msg)]
pub enum FeedPageMsg {
    SetFeed(Box<dyn Iterator<Item = AnyVideo>>),
}

pub struct FeedPageModel {
    app_stream: StreamHandle<AppMsg>,
}

#[widget]
impl Widget for FeedPage {
    fn model(_: &Relm<Self>, app_stream: StreamHandle<AppMsg>) -> FeedPageModel {
        FeedPageModel { app_stream }
    }

    fn update(&mut self, event: FeedPageMsg) {
        match event {
            FeedPageMsg::SetFeed(feed) => {
                self.components
                    .feed_list
                    .emit(LazyListMsg::SetListElementBuilder(Box::new(
                        FeedElementBuilder::new(feed, self.model.app_stream.clone()),
                    )));
            }
        }
    }

    view! {
        gtk::Box {
            orientation: Vertical,
            #[name="feed_list"]
            LazyList<FeedListItem>
        }
    }
}
