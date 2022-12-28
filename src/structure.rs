pub type DoseRange = std::ops::Range<f64>;

#[derive(Debug)]
pub struct Substance {
    pub name: String,
    pub cross_tolerances: Vec<String>,
    pub routes_of_administration: Vec<RoutesOfAdministration>,
    pub uncertain_interactions: Vec<UncertainInteraction>,
    pub unsafe_interactions: Vec<UnsafeInteraction>,
    pub dangerous_interactions: Vec<DangerousInteraction>,
}

#[derive(Debug)]
pub struct RoutesOfAdministration {
    pub name: String,
    pub dose: Dose,
    pub duration: Duration,
}

#[derive(Debug, Default)]
pub struct Dose {
    pub units: DoseUnits,
    pub threshold: Option<f64>,
    pub heavy: Option<f64>,
    pub common: Option<DoseRange>,
    pub light: Option<DoseRange>,
    pub strong: Option<DoseRange>,
}

#[derive(Debug)]
pub enum DoseUnits {
    Mg,
    Ug,
    G,
    Invalid,
}

impl Default for DoseUnits {
    fn default() -> Self {
        DoseUnits::Invalid
    }
}

impl From<String> for DoseUnits {
    fn from(s: String) -> Self {
        match &*s.to_lowercase() {
            "mg" => DoseUnits::Mg,
            "µg" => DoseUnits::Ug,
            "g" => DoseUnits::G,
            _ => DoseUnits::Invalid
        }
    }
}

#[derive(Debug)]
pub enum TimeUnits {
    Minutes,
    Hours,
    Invalid
}

impl From<String> for TimeUnits {
    fn from(s: String) -> Self {
        match &*s.to_lowercase() {
            "hours" => TimeUnits::Hours,
            "minutes" => TimeUnits::Minutes,
            _ => TimeUnits::Invalid,
        }
    }
}

impl Default for TimeUnits {
    fn default() -> Self {
        TimeUnits::Invalid
    }
}

#[derive(Debug, Default)]
pub struct Duration {
    pub afterglow: Option<DoseTimeRange>,
    pub comeup: Option<DoseTimeRange>,
    pub duration: Option<DoseTimeRange>,
    pub offset: Option<DoseTimeRange>,
    pub onset: Option<DoseTimeRange>,
    pub peak: Option<DoseTimeRange>,
    pub total: Option<DoseTimeRange>,
}

#[derive(Debug, Default)]
pub struct DoseTimeRange {
    pub duration: std::time::Duration,
    pub start: f64,
    pub end: f64,
    pub units: TimeUnits,
}

#[derive(Debug)]
pub struct UncertainInteraction {
    pub name: String,
}

#[derive(Debug)]
pub struct UnsafeInteraction {
    pub name: String,
}

#[derive(Debug)]
pub struct DangerousInteraction {
    pub name: String,
}
