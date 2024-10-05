// Config file

// Specify track info to connect to live timing
pub struct TrackConfig {
    pub url_track: &'static str,
}

impl TrackConfig {
    pub fn from_profile(profile: &str) -> TrackConfig {
        match profile {

            // Lignano Circuit
            "LIGNANO-PRACTICE" | "LIGNANO-RACE" => {
                TrackConfig {
                    url_track: "ws://www.apex-timing.com:8182/",
                }
            }
            // Karting Club Los Santos
            "SANTOS-PRACTICE" | "SANTOS-RACE" => {
                TrackConfig {
                    url_track: "ws://www.apex-timing.com:8092/",
                }
            }
            // Karting Center Campillos
            "CAMPILLOS-PRACTICE" | "CAMPILLOS-RACE" | "CAMPILLOS-8HORAS-FP" | "CAMPILLOS-30HORAS" => {
                TrackConfig {
                    url_track: "ws://www.apex-timing.com:8802/",
                }
            }
            // Ariza Racing Circuit
            "ARIZA-PRACTICE" | "ARIZA-RACE" => {
                TrackConfig {
                    url_track: "ws://www.apex-timing.com:8972/",
                }
            }
            _ => panic!("Unknown track: {}", profile),
        }
    }

    pub const fn url_track(&self) -> &'static str {
        self.url_track
    }
}

// Specify all data field to retrieve from live timing
#[derive(Default)]
pub struct RaceConfig {
    pub kart: &'static str,
    pub driver: &'static str,
    pub position: &'static str,
    pub best: &'static str,
    pub last: &'static str,
    pub gap: &'static str,
    pub lap: &'static str,
    pub ontrack: &'static str,
    pub pit: &'static str,
}

impl RaceConfig {
    pub fn from_profile(profile: &str) -> RaceConfig {
        match profile {

            // Free Practice @ Lignano Circuit
            "LIGNANO-PRACTICE" | "SANTOS-PRACTICE" => RaceConfig {
                kart: "c5",
                driver: "c6",
                position: "c4",
                best: "c9",
                last: "c10",
                gap: "c11",
                lap: "c12",
                ontrack: "c13",
                pit: "not-present",
            },
            // Race @ Lignano Circuit
            "LIGNANO-RACE" => RaceConfig {
                kart: "c3",
                driver: "c4",
                position: "c2",
                best: "c8",
                last: "c6",
                gap: "c7",
                lap: "c12",
                ontrack: "c9",
                pit: "c10",
            },
            // Race @ Karting Club Los Santos
            "SANTOS-RACE" => RaceConfig {
                kart: "c3",
                driver: "c4",
                position: "c2",
                best: "c9",
                last: "c5",
                gap: "c7",
                lap: "c6",
                ontrack: "c11",
                pit: "c12",
            },
            // Free Practice @ Karting Center Campillos
            "CAMPILLOS-PRACTICE" => RaceConfig {
                kart: "c4",
                driver: "c5",
                position: "c3",
                best: "c10",
                last: "c9",
                gap: "c11",
                lap: "c13",
                ontrack: "not-present",
                pit: "not-present",
            },
            // Race @ Karting Center Campillos
            "CAMPILLOS-RACE" => RaceConfig {
                kart: "c3",
                driver: "c2",
                position: "c1",
                best: "c10",
                last: "c7",
                gap: "c8",
                lap: "c12",
                ontrack: "not-present",
                pit: "not-present",
            },
            // Free Practice / Race 8 Horas @ Karting Center Campillos
            "CAMPILLOS-8HORAS-FP" => RaceConfig {
                kart: "c6",
                driver: "c2",
                position: "c1",
                best: "c7",
                last: "c5",
                gap: "c4",
                lap: "c12",
                ontrack: "not-present",
                pit: "c3",
            },
            // Race 30 Horas @ Karting Center Campillos
            "CAMPILLOS-30HORAS" => RaceConfig {
                kart: "c4",
                driver: "c5",
                position: "c2",
                best: "c12",
                last: "c10",
                gap: "c11",
                lap: "c6",
                ontrack: "c13",
                pit: "c14",
            },
            // Free Practice / Race @ Ariza Racing Circuit
            "ARIZA-PRACTICE" | "ARIZA-RACE" => RaceConfig {
                kart: "c4",
                driver: "c5",
                position: "c3",
                best: "c7",
                last: "c6",
                gap: "c8",
                lap: "c10",
                ontrack: "not-present",
                pit: "not-present",
            },
            _ => panic!("Unknown profile: {}", profile),
        }
    }

    pub const fn kart(&self) -> &'static str {
        self.kart
    }

    pub const fn driver(&self) -> &'static str {
        self.driver
    }

    pub const fn position(&self) -> &'static str {
        self.position
    }

    pub const fn best(&self) -> &'static str {
        self.best
    }

    pub const fn last(&self) -> &'static str {
        self.last
    }

    pub const fn gap(&self) -> &'static str {
        self.gap
    }

    pub const fn lap(&self) -> &'static str {
        self.lap
    }

    pub const fn ontrack(&self) -> &'static str {
        self.ontrack
    }

    pub const fn pit(&self) -> &'static str {
        self.pit
    }
}