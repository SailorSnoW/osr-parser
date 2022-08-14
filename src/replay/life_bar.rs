use crate::error::Error;
use std::str::FromStr;

#[derive(Default)]
pub struct LifeBar(Vec<LifeBarEvent>);

impl LifeBar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &Vec<LifeBarEvent> {
        &self.0
    }

    pub fn serialize(&self) -> String {
        self.into()
    }
}

impl FromStr for LifeBar {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splitted_events: Vec<&str> = s.split('|').collect();

        let mut events: Vec<LifeBarEvent> = Vec::new();

        for event in splitted_events.iter() {
            if event.len() > 0 && event.contains(',') {
                events.push(LifeBarEvent::from_str(event)?)
            }
        }

        Ok(Self { 0: events })
    }
}

impl From<&LifeBar> for String {
    fn from(life_bar: &LifeBar) -> Self {
        let mut s = String::from("|");

        for event in life_bar.0.iter() {
            let serialized = event.serialize();
            s.push_str(&serialized);
            s.push('|');
        }

        s
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
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
        let valid_events = "|1,2657|1,10213|";

        let life_bar = LifeBar::from_str(valid_events).unwrap();

        assert_eq!(life_bar.0[0].u, 2657);
        assert_eq!(life_bar.0[0].v, 1.0);
        assert_eq!(life_bar.0[1].u, 10213);
        assert_eq!(life_bar.0[1].v, 1.0);
    }
    #[test]
    fn serialize_lifebar() {
        let mut life_bar = LifeBar::new();
        life_bar.0.push(LifeBarEvent { u: 2657, v: 1.0 });
        life_bar.0.push(LifeBarEvent { u: 10213, v: 1.0 });

        let serialized_lifebar = life_bar.serialize();

        assert_eq!(serialized_lifebar, "|1,2657|1,10213|");
    }
}
