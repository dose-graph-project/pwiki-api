pub type DoseRange = std::ops::Range<f64>;

#[derive(Debug)]
pub struct Substance {
    pub name: String,
    pub cross_tolerances: Vec<String>,
    pub routes_of_administration: Vec<RouteOfAdministration>,
    pub uncertain_interactions: Vec<UncertainInteraction>,
    pub unsafe_interactions: Vec<UnsafeInteraction>,
    pub dangerous_interactions: Vec<DangerousInteraction>,
}

impl Substance {
    pub fn dosage_type(&self, dosage: Dosage, roa: ROAs) -> Option<DosageType> {
        let route_of_administration = self.routes_of_administration.iter().find(|i| i.ty == roa);

        if let Some(roa) = route_of_administration {
            Some(roa.dosage_type(dosage))
        } else {
            None
        }
    }

    pub fn route_of_administration(&self, roa: ROAs) -> Option<RouteOfAdministration> {
        self.routes_of_administration
            .iter()
            .find(|i| i.ty == roa)
            .map(|i| i.to_owned())
    }
}

#[derive(Debug, Clone)]
pub struct RouteOfAdministration {
    pub ty: ROAs,
    pub dose_metadata: DoseMetadata,
    pub duration: Duration,
}

impl RouteOfAdministration {
    /// time since start is hours
    pub fn calc_effect(&self, dosage: Dosage, mut time_since_start: f64) -> f64 {
        let dosage_type = self.dosage_type(dosage);

        if let DosageType::BelowThreshold = dosage_type {
            return 0f64;
        }

        if let Some(onset) = &self.duration.onset {
            if time_since_start <= onset.as_hours().midpoint() {
                return 0f64;
            }

            time_since_start -= onset.as_hours().midpoint();
        }

        if let Some(comeup) = &self.duration.comeup {
            if time_since_start <= comeup.as_hours().midpoint() {
                return lerp(0.0, 1.0, time_since_start / comeup.as_hours().midpoint());
            }

            time_since_start -= comeup.as_hours().midpoint();
        }

        if let Some(peak) = &self.duration.peak {
            if time_since_start <= peak.as_hours().midpoint() {
                return 1f64;
            }

            time_since_start -= peak.as_hours().midpoint();
        }

        if let Some(offset) = &self.duration.offset {
            if time_since_start <= offset.as_hours().midpoint() {
                return lerp(1.0, 0.0, time_since_start / offset.as_hours().midpoint());
            }

            time_since_start -= offset.as_hours().midpoint();
        }

        // todo: show this _somehow_
        // if let Some(afterglow) = &self.duration.afterglow {
        //     if time_since_start <= afterglow.as_hours().midpoint() {
        //         return lerp(0.25, 0.0, time_since_start / afterglow.as_hours().midpoint())
        //     }
        // }

        0f64
    }
}

fn lerp(f1: f64, f2: f64, t: f64) -> f64 {
    f1 * (1.0 - t) + f2 * t
}

pub struct Dosage {
    pub units: DoseUnits,
    pub amount: f64,
}

impl Dosage {
    pub fn new(amount: f64, units: DoseUnits) -> Self {
        Self { units, amount }
    }

    pub fn set_amount(&mut self, amount: f64) {
        self.amount = amount;
    }

    pub fn set_units(&mut self, units: DoseUnits) {
        self.units = units;
    }

    /// normalise lhs units to rhs
    pub fn normalise_to_units(&mut self, units: DoseUnits) {
        match (&self.units, &units) {
            (DoseUnits::Mg, DoseUnits::G) | (DoseUnits::Ug, DoseUnits::Mg) => {
                self.set_amount(self.amount / 1e3)
            }
            (DoseUnits::G, DoseUnits::Mg) | (DoseUnits::Mg, DoseUnits::Ug) => {
                self.set_amount(self.amount * 1e3)
            }
            (DoseUnits::Ug, DoseUnits::G) => self.set_amount(self.amount / 1e6),
            (DoseUnits::G, DoseUnits::Ug) => self.set_amount(self.amount * 1e6),
            _ => return,
        }

        self.set_units(units);
    }
}

impl RouteOfAdministration {
    pub fn dosage_type(&self, mut dosage: Dosage) -> DosageType {
        dosage.normalise_to_units(self.dose_metadata.units);

        if let Some(heavy) = self.dose_metadata.heavy {
            if dosage.amount >= heavy {
                return DosageType::Heavy;
            }
        }

        if let Some(threshold) = self.dose_metadata.threshold {
            if dosage.amount == threshold {
                return DosageType::Threshold;
            }
        }

        if let Some(light) = self.dose_metadata.light.clone() {
            if light.contains(&dosage.amount) {
                return DosageType::Light;
            }
        }

        if let Some(common) = self.dose_metadata.common.clone() {
            if common.contains(&dosage.amount) {
                return DosageType::Common;
            }
        }

        if let Some(strong) = self.dose_metadata.strong.clone() {
            if strong.contains(&dosage.amount) {
                return DosageType::Strong;
            }
        }

        DosageType::BelowThreshold
    }
}

#[derive(Debug)]
pub enum DosageType {
    Threshold,
    Heavy,
    Common,
    Light,
    Strong,
    BelowThreshold,
}

#[derive(Debug, Default, Clone)]
pub struct DoseMetadata {
    pub units: DoseUnits,
    pub threshold: Option<f64>,
    pub heavy: Option<f64>,
    pub common: Option<DoseRange>,
    pub light: Option<DoseRange>,
    pub strong: Option<DoseRange>,
}

#[derive(Debug, Clone, Copy)]
pub enum DoseUnits {
    Mg,
    Ml,
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
            "ml" => DoseUnits::Ml,
            _ => DoseUnits::Invalid,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimeUnits {
    Minutes,
    Hours,
    Seconds,
    Invalid,
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

#[derive(Debug, Default, Clone)]
pub struct Duration {
    pub afterglow: Option<DoseTimeRange>,
    pub comeup: Option<DoseTimeRange>,
    pub duration: Option<DoseTimeRange>,
    pub offset: Option<DoseTimeRange>,
    pub onset: Option<DoseTimeRange>,
    pub peak: Option<DoseTimeRange>,
    pub total: Option<DoseTimeRange>,
}

#[derive(Debug, Default, Clone)]
pub struct DoseTimeRange {
    pub duration: std::time::Duration,
    pub start: f64,
    pub end: f64,
    pub midpoint: f64,
    pub units: TimeUnits,
}

impl DoseTimeRange {
    pub fn set_units(&mut self, units: TimeUnits) {
        self.units = units;
    }

    pub fn set_start(&mut self, start: f64) {
        self.start = start;
    }

    pub fn set_end(&mut self, end: f64) {
        self.end = end;
    }

    pub fn set_midpoint(&mut self, midpoint: f64) {
        self.midpoint = midpoint;
    }

    pub fn recalc_midpoint(&mut self) {
        self.set_midpoint((self.start + self.end) / 2.0);
    }

    pub fn midpoint(&self) -> f64 {
        (self.start + self.end) / 2.0
    }

    pub fn as_hours(&self) -> DoseTimeRange {
        match &self.units {
            TimeUnits::Minutes => DoseTimeRange {
                duration: self.duration,
                start: self.start / 60.0,
                end: self.end / 60.0,
                midpoint: self.midpoint / 60.0,
                units: TimeUnits::Hours,
            },
            TimeUnits::Seconds => DoseTimeRange {
                duration: self.duration,
                start: self.start / 3600.0,
                end: self.end / 3600.0,
                midpoint: self.midpoint / 3600.0,
                units: TimeUnits::Hours,
            },
            TimeUnits::Hours => self.to_owned(),

            _ => unimplemented!(),
        }
    }

    pub fn as_minutes(&self) -> DoseTimeRange {
        match &self.units {
            TimeUnits::Seconds => DoseTimeRange {
                duration: self.duration,
                start: self.start / 60.0,
                end: self.end / 60.0,
                midpoint: self.midpoint / 60.0,
                units: TimeUnits::Hours,
            },
            TimeUnits::Hours => DoseTimeRange {
                duration: self.duration,
                start: self.start * 60.0,
                end: self.end * 60.0,
                midpoint: self.midpoint * 60.0,
                units: TimeUnits::Hours,
            },
            TimeUnits::Minutes => self.to_owned(),

            _ => unimplemented!(),
        }
    }

    pub fn to_hours(&mut self) {
        match &self.units {
            TimeUnits::Minutes => {
                self.set_start(self.start / 60.0);
                self.set_end(self.end / 60.0);
                self.recalc_midpoint();
            }
            TimeUnits::Seconds => {
                self.set_start(self.start / 3600.0);
                self.set_end(self.end / 3600.0);
                self.recalc_midpoint();
            }
            _ => return,
        }
    }

    pub fn to_minutes(&mut self) {
        match &self.units {
            TimeUnits::Hours => {
                self.set_start(self.start * 60.0);
                self.set_end(self.end * 60.0);
                self.recalc_midpoint();
            }
            TimeUnits::Seconds => {
                self.set_start(self.start / 60.0);
                self.set_end(self.end / 60.0);
                self.recalc_midpoint();
            }

            _ => return,
        }
    }

    pub fn to_seconds(&mut self) {
        match &self.units {
            TimeUnits::Hours => {
                self.set_start(self.start * 3600.0);
                self.set_end(self.end * 3600.0);
                self.recalc_midpoint();
            }
            TimeUnits::Minutes => {
                self.set_start(self.start * 60.0);
                self.set_end(self.end * 60.0);
                self.recalc_midpoint();
            }

            _ => return,
        }
    }

    /// normalise lhs units to rhs
    pub fn normalise_to_units(&mut self, units: TimeUnits) {
        match (&self.units, &units) {
            (.., TimeUnits::Hours) => self.to_hours(),
            (.., TimeUnits::Minutes) => self.to_minutes(),
            (.., TimeUnits::Seconds) => self.to_seconds(),

            _ => return,
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// RoutesOfAdministration
pub enum ROAs {
    Oral,
    Sublingual,
    Buccal,
    Insuffilation,
    Inhalation,
    Smoked,
    Vaporised,
    Intravenous,
    Intramuscular,
    Subcutaneous,
    Rectal,
    Transdermal,
    Invalid,
}

impl From<String> for ROAs {
    fn from(s: String) -> Self {
        match &*s.to_lowercase() {
            "oral" => Self::Oral,
            "sublingual" => Self::Sublingual,
            "buccal" => Self::Buccal,
            "insuffilation" => Self::Insuffilation,
            "inhalation" => Self::Inhalation,
            "smoked" => Self::Smoked,
            "vaporised" => Self::Vaporised,
            "intravenous" => Self::Intravenous,
            "intramuscular" => Self::Intramuscular,
            "subcutaneous" => Self::Subcutaneous,
            "rectal" => Self::Rectal,
            "transdermal" => Self::Transdermal,
            _ => Self::Invalid,
        }
    }
}

impl Default for ROAs {
    fn default() -> Self {
        Self::Invalid
    }
}

#[cfg(test)]
mod test {
    use crate::query::SubstanceQuery;

    use super::*;

    #[tokio::test]
    async fn test_dosage_type() {
        let data = SubstanceQuery::substance_data("LSD").await.unwrap();
        let dosage_type = data[0].dosage_type(Dosage::new(100.0, DoseUnits::Ug), ROAs::Sublingual);

        dbg!(dosage_type);
    }
}
