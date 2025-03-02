use crate::error::Error;
use std::str::FromStr;

/// Represents parsed data of the life bar graph
#[derive(Default, Debug)]
pub struct LifeBar {
    pub base_time: u32,
    events: Vec<LifeBarEvent>,
}

impl LifeBar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &Vec<LifeBarEvent> {
        &self.events
    }

    pub fn parse(str: &str) -> Result<Self, Error> {
        LifeBar::from_str(str)
    }

    pub fn serialize(&self) -> String {
        self.into()
    }

    pub fn delete_bar_data(&mut self) {
        *self = Self::default()
    }
}

impl FromStr for LifeBar {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splitted_events: Vec<&str> = s.split('|').collect();

        let mut events: Vec<LifeBarEvent> = Vec::new();

        for event in splitted_events.iter() {
            if !event.is_empty() && event.contains(',') {
                if let Ok(e) = LifeBarEvent::from_str(event) {
                    events.push(e)
                }
            }
        }

        Ok(Self {
            base_time: u32::from_str(splitted_events[0]).unwrap_or_default(),
            events,
        })
    }
}

impl From<&LifeBar> for String {
    fn from(life_bar: &LifeBar) -> Self {
        if life_bar.events().is_empty() {
            return String::from("");
        }

        let mut s = String::new();

        if life_bar.base_time > 0 {
            s.push_str(&life_bar.base_time.to_string())
        }

        for event in life_bar.events.iter() {
            s.push('|');
            let serialized = event.serialize();
            s.push_str(&serialized);
        }

        if !s.is_empty() {
            s.push('|')
        }

        s.push_str("1,");

        s
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug, Copy, Clone)]
pub struct LifeBarEvent {
    /// time in milliseconds into the song
    pub u: u32,
    /// floating point value from 0 - 1 that represents the amount of life you have at the `u` time
    /// (0 = life bar is empty, 1= life bar is full)
    pub v: f32,
}

impl LifeBarEvent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn serialize(&self) -> String {
        self.into()
    }
}

impl FromStr for LifeBarEvent {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splitted_event: Vec<&str> = s.split(',').collect();

        if splitted_event.len() != 2 {
            return Err(Error::InvalidStringFrameFormat);
        };

        Ok(Self {
            u: u32::from_str(splitted_event[1]).map_err(|_| Error::CantParseFrameValue)?,
            v: f32::from_str(splitted_event[0]).map_err(|_| Error::CantParseFrameValue)?,
        })
    }
}

impl From<&LifeBarEvent> for String {
    fn from(event: &LifeBarEvent) -> Self {
        format!("{},{}", event.v, event.u)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    // Life Bar Event

    #[test]
    fn parse_event() {
        let valid_event_str = "1,2249";

        let event = LifeBarEvent::from_str(valid_event_str).unwrap();

        assert_eq!(event.u, 2249_u32);
        assert_eq!(event.v, 1.0);
    }
    #[test]
    fn serialize_event() {
        let event = LifeBarEvent { u: 2249, v: 1.0 };

        let serialized_event: String = event.serialize();

        assert_eq!(serialized_event, "1,2249");
    }

    // Life Bar

    #[test]
    fn parse_lifebar() {
        let valid_events = "256|1,2657|1,10213|1,";

        let life_bar = LifeBar::from_str(valid_events).unwrap();

        assert_eq!(life_bar.base_time, 256);

        assert_eq!(life_bar.events[0].u, 2657);
        assert_eq!(life_bar.events[0].v, 1.0);
        assert_eq!(life_bar.events[1].u, 10213);
        assert_eq!(life_bar.events[1].v, 1.0);
    }
    #[test]
    fn serialize_lifebar() {
        let mut life_bar = LifeBar::new();
        life_bar.base_time = 256;
        life_bar.events.push(LifeBarEvent { u: 2657, v: 1.0 });
        life_bar.events.push(LifeBarEvent { u: 10213, v: 1.0 });

        let serialized_lifebar = life_bar.serialize();

        assert_eq!(serialized_lifebar, "256|1,2657|1,10213|1,");
    }
}
