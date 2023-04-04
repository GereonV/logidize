//! Sensible [ChannelFilterMap]s.

use std::{collections::btree_map::{BTreeMap, Entry}, fmt::Display, ptr::NonNull};

use crate::loggers::{Level, LogObject};

/// A trait for displaying channels of [LogObject]s or discarding them.
pub trait ChannelFilterMap {
    /// The [Display] for channels of [LogObject]s that should be logged.
    type DisplayType: Display;

    /// Returns [None] when the [LogObject] should be discarded or
    /// [Some(v)] where `v` is used to display the channel if the [LogObject] should be logged.
    #[must_use]
    fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType>;
}

impl<T: FnMut(&LogObject) -> Option<DisplayType>, DisplayType: Display> ChannelFilterMap for T {
    type DisplayType = DisplayType;

    fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
        self(log_object)
    }
}

/// A [ChannelFilterMap] that logs all channels using their ID.
#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct InvisibleChannelFilterMap;
impl ChannelFilterMap for InvisibleChannelFilterMap {
    type DisplayType = usize;

    fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
        Some(log_object.channel_id)
    }
}

/// A [ChannelFilterMap] that logs all known channels using their name.
#[derive(Clone, Copy, Debug, Hash)]
pub struct StaticChannelFilterMap<T: 'static + Display, const N: usize>(pub &'static [T; N]);
impl<T: 'static + Display, const N: usize> ChannelFilterMap for StaticChannelFilterMap<T, N> {
    type DisplayType = &'static T;

    fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
        self.0.get(log_object.channel_id)
    }
}

/// A [ChannelFilterMap] that logs all known channels using their name when the channel's minimum severity level isn't lower than [LogObject::severity].
#[derive(Clone, Copy, Debug, Hash)]
pub struct StaticSeverityChannelFilterMap<T: 'static + Display, const N: usize>(pub &'static [(T, Level); N]);
impl<T: 'static + Display, const N: usize> ChannelFilterMap for StaticSeverityChannelFilterMap<T, N> {
    type DisplayType = &'static T;

    fn filter_map(&mut self, log_object: &LogObject) -> Option<Self::DisplayType> {
        self.0.get(log_object.channel_id)
            .and_then(|(t, min_severity)| match &log_object.severity < min_severity {
                true => None,
                false => Some(t),
            })
    }
}

// unsafe because it may outlive borrowed data
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BorrowDisplay<T: Display>(NonNull<T>);

impl<T: Display> Display for BorrowDisplay<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = unsafe { self.0.as_ref() };
        t.fmt(f)
    }
}

/// A channel of [SimpleChannelFilterMap].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleChannel<T: Display> {
    /// Whether the channel is enabled. [SimpleChannelFilterMap] won't log from disabled channels.
    pub enabled: bool,
    /// The channel's minimum severity level. [SimpleChannelFilterMap] will filter [LogObject]s with lower severity.
    pub min_severity: Level,
    /// The name (or other representation) of the channel.
    pub name: T,
}

impl<T: Display + Default> Default for SimpleChannel<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: Display> SimpleChannel<T> {
    /// ```
    /// # use logidize::{filter_maps::SimpleChannel, loggers::Level};
    /// let c1 = SimpleChannel::new(0);
    /// let c2 = SimpleChannel { enabled: true, min_severity: Level::DEBUG, name: 0 };
    /// assert_eq!(c1, c2);
    /// ```
    #[must_use]
    pub const fn new(name: T) -> Self {
        Self { enabled: true, min_severity: Level::DEBUG, name }
    }
}

/// [ChannelFilterMap] wrapper over [BTreeMap<usize, SimpleChannel>].
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
    /// See [BTreeMap::new()].
    #[must_use]
    pub const fn new() -> Self {
        Self { channels: BTreeMap::new() }
    }

    /// See [BTreeMap::get()].
    #[must_use]
    pub fn channel(&self, channel_id: usize) -> Option<&SimpleChannel<T>> {
        self.channels.get(&channel_id)
    }

    /// See [BTreeMap::get_mut()].
    #[must_use]
    pub fn channel_mut(&mut self, channel_id: usize) -> Option<&mut SimpleChannel<T>> {
        self.channels.get_mut(&channel_id)
    }

    /// See [BTreeMap::insert()].
    pub fn insert_channel(&mut self, channel_id: usize, channel: SimpleChannel<T>) -> Option<SimpleChannel<T>> {
        self.channels.insert(channel_id, channel)
    }

    /// See [BTreeMap::entry()] and [Entry].
    pub fn modify_or_insert_channel(&mut self, channel_id: usize, f: impl FnOnce(&mut SimpleChannel<T>), channel: SimpleChannel<T>) -> &mut SimpleChannel<T> {
        self.channels.entry(channel_id)
            .and_modify(f)
            .or_insert(channel)
    }

    /// See [BTreeMap::entry()] and [Entry].
    pub fn modify_or_insert_channel_with(&mut self, channel_id: usize, f1: impl FnOnce(&mut SimpleChannel<T>), f2: impl FnOnce() -> SimpleChannel<T>) -> &mut SimpleChannel<T> {
        self.channels.entry(channel_id)
            .and_modify(f1)
            .or_insert_with(f2)
    }

    /// See [BTreeMap::entry()] and [Entry].
    pub fn modify_or_insert_channel_with_id(&mut self, channel_id: usize, f1: impl FnOnce(&mut SimpleChannel<T>), f2: impl FnOnce(&usize) -> SimpleChannel<T>) -> &mut SimpleChannel<T> {
        self.channels.entry(channel_id)
            .and_modify(f1)
            .or_insert_with_key(f2)
    }

    /// See [BTreeMap::get()].
    #[must_use]
    pub fn channel_name(&self, channel_id: usize) -> Option<&T> {
        self.channel(channel_id).map(|channel| &channel.name)
    }

    /// Updates channel's name and returns `(_, Some(previous_name))` or inserts `SimpleChannel::new(name.into())` and returns `(_, None)`.
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

    /// See [BTreeMap::get()].
    #[must_use]
    pub fn channel_enabled(&self, channel_id: usize) -> Option<bool> {
        self.channel(channel_id).map(|channel| channel.enabled)
    }

    /// Sets [SimpleChannel::enabled] if channel exists.
    pub fn set_channel_enabled(&mut self, channel_id: usize, enabled: bool) -> Option<bool> {
        let channel = self.channels.get_mut(&channel_id)?;
        Some(std::mem::replace(&mut channel.enabled, enabled))
    }

    /// See [BTreeMap::get()].
    #[must_use]
    pub fn channel_min_severity(&self, channel_id: usize) -> Option<Level> {
        self.channel(channel_id).map(|channel| channel.min_severity)
    }

    /// Sets [SimpleChannel::min_severity] if channel exists.
    pub fn set_channel_min_severity(&mut self, channel_id: usize, min_severity: Level) -> Option<Level> {
        let channel = self.channels.get_mut(&channel_id)?;
        Some(std::mem::replace(&mut channel.min_severity, min_severity))
    }
}

impl<T: Display + Default> SimpleChannelFilterMap<T> {
    /// See [BTreeMap::entry()] and [Entry].
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
