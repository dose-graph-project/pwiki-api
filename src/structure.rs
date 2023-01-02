#![allow(unused_assignments)]

use std::fmt::Display;

use chrono::{DateTime, Utc};

pub type DoseRange = std::ops::Range<f64>;

#[derive(Debug, Clone)]
pub struct Substance {
    pub name: String,
    pub cross_tolerances: Vec<String>,
    pub routes_of_administration: Vec<RouteOfAdministration>,
    pub uncertain_interactions: Vec<UncertainInteraction>,
    pub unsafe_interactions: Vec<UnsafeInteraction>,
    pub dangerous_interactions: Vec<DangerousInteraction>,
}

impl Substance {
    pub fn new_ingestion(
        &self,
        amount: f64,
        units: DoseUnits,
        timestamp: DateTime<Utc>,
        route_of_administration: ROAs,
    ) -> Ingestion {
        Ingestion::new(
            amount,
            units,
            timestamp,
            route_of_administration,
            self.clone(),
        )
    }

    pub fn dosage_type(&self, dosage: &Ingestion) -> Option<DosageType> {
        let route_of_administration = self
            .routes_of_administration
            .iter()
            .find(|i| i.ty == dosage.route_of_administration);

        route_of_administration.map(|roa| roa.dosage_type(dosage))
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
    pub fn calc_effect(&self, dosage: Ingestion, mut time_since_start: f64) -> f64 {
        let dosage_type = self.dosage_type(&dosage);

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

    pub fn cumulative_total(&self) -> f64 {
        let onset_end = self
            .duration
            .onset
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .end;

        let comeup_end = self
            .duration
            .comeup
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .end
            + onset_end;
        let peak_end = self
            .duration
            .peak
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .end
            + comeup_end;
        let offset_end = self
            .duration
            .offset
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .end
            + peak_end;

        offset_end
    }

    pub fn estimate_points(&self) -> Vec<(f64, f64)> {
        let onset = self
            .duration
            .onset
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .midpoint();
        let comeup = self
            .duration
            .comeup
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .midpoint()
            + onset;
        let peak = self
            .duration
            .peak
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .midpoint()
            + comeup;
        let offset = self
            .duration
            .offset
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .midpoint()
            + peak;

        vec![
            (0f64, 0f64),
            (onset, 0f64),
            (comeup, 1f64),
            (peak, 1f64),
            (offset, 0f64),
        ]
    }

    pub fn comeup_distribution(&self) -> Vec<(f64, f64)> {
        let onset_start = self
            .duration
            .onset
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .start;
        let onset_end = self
            .duration
            .onset
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .end;

        let comeup_start = self
            .duration
            .comeup
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .start
            + onset_start;
        let comeup_end = self
            .duration
            .comeup
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .end
            + onset_end;

        vec![
            (onset_start, 0f64),
            (onset_end, 0f64),
            (comeup_end, 1f64),
            (comeup_start, 1f64),
            (onset_start, 0f64),
        ]
    }

    pub fn offset_distribution(&self) -> Vec<(f64, f64)> {
        let onset_start = self
            .duration
            .onset
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .start;
        let onset_end = self
            .duration
            .onset
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .end;

        let comeup_start = self
            .duration
            .comeup
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .start
            + onset_start;
        let comeup_end = self
            .duration
            .comeup
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .end
            + onset_end;

        let peak_start = self
            .duration
            .peak
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .start
            + comeup_start;
        let peak_end = self
            .duration
            .peak
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .end
            + comeup_end;

        let offset_start = self
            .duration
            .offset
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .start
            + peak_start;
        let offset_end = self
            .duration
            .offset
            .as_ref()
            .unwrap_or(&DoseTimeRange::ZERO)
            .as_seconds()
            .end
            + peak_end;

        vec![
            (peak_start, 1f64),
            (peak_end, 1f64),
            (offset_end, 0f64),
            (offset_start, 0f64),
            (peak_start, 1f64),
        ]
    }
}

fn lerp(f1: f64, f2: f64, t: f64) -> f64 {
    f1 * (1.0 - t) + f2 * t
}

#[derive(Debug, Clone)]
pub struct Ingestion {
    pub units: DoseUnits,
    pub amount: f64,
    pub timestamp: DateTime<Utc>,
    pub route_of_administration: ROAs,
    pub substance: Substance,
}

impl Ingestion {
    pub fn new(
        amount: f64,
        units: DoseUnits,
        timestamp: DateTime<Utc>,
        route_of_administration: ROAs,
        substance: Substance,
    ) -> Self {
        Self {
            units,
            amount,
            timestamp,
            route_of_administration,
            substance,
        }
    }

    pub fn roa(&self) -> RouteOfAdministration {
        self
            .substance
            .route_of_administration(self.route_of_administration)
            .unwrap()
    }

    pub fn dosage_type(&self) -> Option<DosageType> {
        self.substance.dosage_type(self)
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

    pub fn normalise_as_units(&self, units: DoseUnits) -> Self {
        dbg!(&self.units, &units);
        match (&self.units, &units) {
            (DoseUnits::Mg, DoseUnits::G) | (DoseUnits::Ug, DoseUnits::Mg) => Self {
                amount: self.amount / 1e3,
                ..self.clone()
            },
            (DoseUnits::G, DoseUnits::Mg) | (DoseUnits::Mg, DoseUnits::Ug) => Self {
                amount: self.amount * 1e3,
                ..self.clone()
            },
            (DoseUnits::Ug, DoseUnits::G) => Self {
                amount: self.amount / 1e6,
                ..self.clone()
            },
            (DoseUnits::G, DoseUnits::Ug) => Self {
                amount: self.amount * 1e6,
                ..self.clone()
            },
            (l, r) if l == r => {
                self.clone()
            },
            _ => unreachable!()
        }
    }
}

impl RouteOfAdministration {
    pub fn dosage_type(&self, dosage: &Ingestion) -> DosageType {
        dosage.normalise_as_units(self.dose_metadata.units);

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

        if let Some(light) = &self.dose_metadata.light {
            if light.contains(&dosage.amount) {
                return DosageType::Light;
            }
        }

        if let Some(common) = &self.dose_metadata.common {
            if common.contains(&dosage.amount) {
                return DosageType::Common;
            }
        }

        if let Some(strong) = &self.dose_metadata.strong {
            if strong.contains(&dosage.amount) {
                return DosageType::Strong;
            }
        }

        DosageType::BelowThreshold
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

impl Display for DoseUnits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DoseUnits::Mg => f.write_str("mg"),
            DoseUnits::Ml => f.write_str("ml"),
            DoseUnits::Ug => f.write_str("µg"),
            DoseUnits::G => f.write_str("g"),
            DoseUnits::Invalid => todo!(),
        }
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
            "seconds" => TimeUnits::Seconds,
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

#[derive(Debug, Clone)]
pub struct DoseTimeRange {
    pub duration: std::time::Duration,
    pub start: f64,
    pub end: f64,
    pub midpoint: f64,
    pub units: TimeUnits,
}

impl Default for DoseTimeRange {
    fn default() -> Self {
        Self::ZERO
    }
}

impl DoseTimeRange {
    pub const ZERO: DoseTimeRange = DoseTimeRange {
        duration: std::time::Duration::ZERO,
        start: 0f64,
        end: 0f64,
        midpoint: 0f64,
        units: TimeUnits::Hours,
    };

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
                units: TimeUnits::Minutes,
            },
            TimeUnits::Hours => DoseTimeRange {
                duration: self.duration,
                start: self.start * 60.0,
                end: self.end * 60.0,
                midpoint: self.midpoint * 60.0,
                units: TimeUnits::Minutes,
            },
            TimeUnits::Minutes => self.to_owned(),

            _ => unimplemented!(),
        }
    }

    pub fn as_seconds(&self) -> DoseTimeRange {
        match &self.units {
            TimeUnits::Minutes => DoseTimeRange {
                duration: self.duration,
                start: self.start * 60.0,
                end: self.end * 60.0,
                midpoint: self.midpoint * 60.0,
                units: TimeUnits::Seconds,
            },
            TimeUnits::Hours => DoseTimeRange {
                duration: self.duration,
                start: self.start * 3600.0,
                end: self.end * 3600.0,
                midpoint: self.midpoint * 3600.0,
                units: TimeUnits::Seconds,
            },
            TimeUnits::Seconds => self.to_owned(),

            _ => unimplemented!(),
        }
    }

    pub fn to_hours(&mut self) {
        match &self.units {
            TimeUnits::Minutes => {
                self.set_start(self.start / 60.0);
                self.set_end(self.end / 60.0);
                self.recalc_midpoint();
                self.set_units(TimeUnits::Hours);
            }
            TimeUnits::Seconds => {
                self.set_start(self.start / 3600.0);
                self.set_end(self.end / 3600.0);
                self.recalc_midpoint();
                self.set_units(TimeUnits::Hours);
            }
            _ => (),
        }
    }

    pub fn to_minutes(&mut self) {
        match &self.units {
            TimeUnits::Hours => {
                self.set_start(self.start * 60.0);
                self.set_end(self.end * 60.0);
                self.recalc_midpoint();
                self.set_units(TimeUnits::Minutes);
            }
            TimeUnits::Seconds => {
                self.set_start(self.start / 60.0);
                self.set_end(self.end / 60.0);
                self.recalc_midpoint();
                self.set_units(TimeUnits::Minutes);
            }

            _ => (),
        }
    }

    pub fn to_seconds(&mut self) {
        match &self.units {
            TimeUnits::Hours => {
                self.set_start(self.start * 3600.0);
                self.set_end(self.end * 3600.0);
                self.recalc_midpoint();
                self.set_units(TimeUnits::Seconds);
            }
            TimeUnits::Minutes => {
                self.set_start(self.start * 60.0);
                self.set_end(self.end * 60.0);
                self.recalc_midpoint();
                self.set_units(TimeUnits::Seconds);
            }

            _ => (),
        }
    }

    /// normalise lhs units to rhs
    pub fn normalise_to_units(&mut self, units: TimeUnits) {
        match (&self.units, &units) {
            (.., TimeUnits::Hours) => self.to_hours(),
            (.., TimeUnits::Minutes) => self.to_minutes(),
            (.., TimeUnits::Seconds) => self.to_seconds(),

            _ => (),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UncertainInteraction {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct UnsafeInteraction {
    pub name: String,
}

#[derive(Debug, Clone)]
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
        let ingestion = data[0].new_ingestion(100.0, DoseUnits::Ug, Utc::now(), ROAs::Sublingual);
        let dosage_type = ingestion.dosage_type();
        assert_eq!(dosage_type.unwrap(), DosageType::Common);
    }
}
