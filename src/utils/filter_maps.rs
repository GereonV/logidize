use std::{collections::btree_map::{BTreeMap, Entry}, fmt::Display, ptr::NonNull};

use crate::{Level, LogObject};

pub trait ChannelFilterMap {
    type DisplayType: Display;

    fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType>;
}

impl<T: FnMut(&LogObject) -> Option<DisplayType>, DisplayType: Display> ChannelFilterMap for T {
    type DisplayType = DisplayType;

    fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
        self(log_object)
    }
}

#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct InvisibleChannelFilterMap;
impl ChannelFilterMap for InvisibleChannelFilterMap {
    type DisplayType = usize;

    fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
        Some(log_object.channel_id)
    }
}

// unsafe because it may outlive borrowed data
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BorrowDisplay<T: Display>(NonNull<T>);

impl<T: Display> Display for BorrowDisplay<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = unsafe { self.0.as_ref() };
        t.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleChannel<T: Display> {
    pub enabled: bool,
    pub min_severity: Level,
    pub name: T,
}

impl<T: Display + Default> Default for SimpleChannel<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: Display> SimpleChannel<T> {
    pub const fn new(name: T) -> Self {
        Self { enabled: true, min_severity: Level::DEBUG, name }
    }
}

#[derive(Clone, Debug)]
pub struct SimpleChannelFilterMap<T: Display> {
    channels: BTreeMap<usize, SimpleChannel<T>>,
}

impl<T: Display> Default for SimpleChannelFilterMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Display> SimpleChannelFilterMap<T> {
    pub const fn new() -> Self {
        Self { channels: BTreeMap::new() }
    }

    pub fn channel(&self, channel_id: usize) -> Option<&SimpleChannel<T>> {
        self.channels.get(&channel_id)
    }

    pub fn channel_mut(&mut self, channel_id: usize) -> Option<&mut SimpleChannel<T>> {
        self.channels.get_mut(&channel_id)
    }

    pub fn insert_channel(&mut self, channel_id: usize, channel: SimpleChannel<T>) -> Option<SimpleChannel<T>> {
        self.channels.insert(channel_id, channel)
    }

    pub fn modify_or_insert_channel(&mut self, channel_id: usize, f: impl FnOnce(&mut SimpleChannel<T>), channel: SimpleChannel<T>) -> &mut SimpleChannel<T> {
        self.channels.entry(channel_id)
            .and_modify(f)
            .or_insert(channel)
    }

    pub fn modify_or_insert_channel_with(&mut self, channel_id: usize, f1: impl FnOnce(&mut SimpleChannel<T>), f2: impl FnOnce() -> SimpleChannel<T>) -> &mut SimpleChannel<T> {
        self.channels.entry(channel_id)
            .and_modify(f1)
            .or_insert_with(f2)
    }

    pub fn modify_or_insert_channel_with_id(&mut self, channel_id: usize, f1: impl FnOnce(&mut SimpleChannel<T>), f2: impl FnOnce(&usize) -> SimpleChannel<T>) -> &mut SimpleChannel<T> {
        self.channels.entry(channel_id)
            .and_modify(f1)
            .or_insert_with_key(f2)
    }

    pub fn channel_name(&self, channel_id: usize) -> Option<&T> {
        self.channel(channel_id).map(|channel| &channel.name)
    }

    pub fn set_channel_name_or_insert_channel(&mut self, channel_id: usize, name: impl Into<T>) -> (&mut SimpleChannel<T>, Option<T>) {
        match self.channels.entry(channel_id) {
            Entry::Occupied(e) => {
                let channel = e.into_mut();
                let name = std::mem::replace(&mut channel.name, name.into());
                (channel, Some(name))
            },
            Entry::Vacant(e) => {
                (e.insert(SimpleChannel::new(name.into())), None)
            },
        }
    }

    pub fn channel_enabled(&self, channel_id: usize) -> Option<bool> {
        self.channel(channel_id).map(|channel| channel.enabled)
    }

    pub fn set_channel_enabled(&mut self, channel_id: usize, enabled: bool) -> Option<bool> {
        let channel = self.channels.get_mut(&channel_id)?;
        Some(std::mem::replace(&mut channel.enabled, enabled))
    }

    pub fn channel_min_severity(&self, channel_id: usize) -> Option<Level> {
        self.channel(channel_id).map(|channel| channel.min_severity)
    }

    pub fn set_channel_min_severity(&mut self, channel_id: usize, min_severity: Level) -> Option<Level> {
        let channel = self.channels.get_mut(&channel_id)?;
        Some(std::mem::replace(&mut channel.min_severity, min_severity))
    }
}

impl<T: Display + Default> SimpleChannelFilterMap<T> {
    pub fn modify_or_default(&mut self, channel_id: usize, f: impl FnOnce(&mut SimpleChannel<T>)) -> &mut SimpleChannel<T> {
        self.channels.entry(channel_id)
            .and_modify(f)
            .or_default()
    }
}

impl<T: Display> ChannelFilterMap for SimpleChannelFilterMap<T> {
    type DisplayType = BorrowDisplay<T>;

    fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
        let channel = self.channels.get(&log_object.channel_id)?;
        if !channel.enabled || log_object.severity < channel.min_severity {
            return None;
        }
        let ptr = &channel.name as *const T;
        let ptr = unsafe {
            NonNull::new_unchecked(ptr as *mut T)
        };
        Some(BorrowDisplay(ptr))
    }
}
