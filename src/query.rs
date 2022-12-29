use graphql_client::GraphQLQuery;

use crate::error::ApiError;
use crate::structure::{
    DangerousInteraction, DoseMetadata, DoseTimeRange, Duration, RouteOfAdministration, Substance,
    TimeUnits, UncertainInteraction, UnsafeInteraction,
};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "src/wiki_api.graphql",
    response_derives = "Serialize,PartialEq,Debug,Clone"
)]
pub struct SubstanceQuery;

macro_rules! vec_impl {
    ( $($ty:ty),+ $(,)? ) => {
        $(
            impl UnwrapVec<$ty>
                for Vec<Option<$ty>> {
                    fn unwrap(self) -> Vec<$ty> {
                        self.into_iter().filter_map(|u| u).collect()
                    }
                }
        )*
    };
}

impl SubstanceQuery {
    async fn substance_data_internal(
        substance: impl AsRef<str>,
    ) -> Result<graphql_client::Response<substance_query::ResponseData>, Box<dyn std::error::Error>>
    {
        // this is the important line
        let request_body = SubstanceQuery::build_query(substance_query::Variables {
            substance: substance.as_ref().into(),
        });

        let client = reqwest::Client::new();
        let res = client
            .post("https://api.psychonautwiki.org/")
            .json(&request_body)
            .send()
            .await?;
        let response_body: graphql_client::Response<substance_query::ResponseData> =
            res.json().await?;

        if let Some(e) = response_body.errors {
            let messages = e.into_iter().map(|e| e.message).collect();
            return Err(Box::new(ApiError::new(messages)));
        }

        Ok(response_body)
    }

    pub async fn substance_data(
        substance: impl AsRef<str>,
    ) -> Result<Vec<Substance>, Box<dyn std::error::Error>>
    {
        let r = SubstanceQuery::substance_data_internal(substance).await?;
        let s = r.data.and_then(|o| o.substances).ok_or_else(|| {
            Box::new(ApiError::new(vec![
                "Missing substance data.".to_string()
            ]))
        })?;

        Ok(s.into_iter().filter_map(|i| i).map(|i| Substance::from(i)).collect::<Vec<_>>())
    }
}

trait GetSubstances {
    fn substances(self) -> Vec<substance_query::SubstanceQuerySubstances>;
}

impl GetSubstances for substance_query::ResponseData {
    fn substances(self) -> Vec<substance_query::SubstanceQuerySubstances> {
        self.substances.unwrap().unwrap()
    }
}

trait UnwrapVec<T> {
    fn unwrap(self) -> Vec<T>;
}

vec_impl! {
    substance_query::SubstanceQuerySubstances,
    substance_query::SubstanceQuerySubstancesRoas,
    substance_query::SubstanceQuerySubstancesUncertainInteractions,
    substance_query::SubstanceQuerySubstancesUnsafeInteractions,
    substance_query::SubstanceQuerySubstancesDangerousInteractions,
}

impl From<crate::query::substance_query::SubstanceQuerySubstances> for Substance {
    fn from(substance_query: crate::query::substance_query::SubstanceQuerySubstances) -> Substance {
        Substance {
            name: substance_query.name.unwrap_or_default(),
            cross_tolerances: substance_query
                .cross_tolerances
                .unwrap_or_default()
                .into_iter()
                .filter_map(|i| i)
                .collect(),
            dangerous_interactions: substance_query
                .dangerous_interactions
                .unwrap_or_default()
                .into_iter()
                .filter_map(|i| i)
                .map(|i| i.into())
                .collect(),
            routes_of_administration: substance_query
                .roas
                .unwrap_or_default()
                .into_iter()
                .filter_map(|i| i)
                .map(|i| i.into())
                .collect(),
            uncertain_interactions: substance_query
                .uncertain_interactions
                .unwrap_or_default()
                .into_iter()
                .filter_map(|i| i)
                .map(|i| i.into())
                .collect(),
            unsafe_interactions: substance_query
                .unsafe_interactions
                .unwrap_or_default()
                .into_iter()
                .filter_map(|i| i)
                .map(|i| i.into())
                .collect(),
        }
    }
}

impl From<crate::query::substance_query::SubstanceQuerySubstancesDangerousInteractions>
    for DangerousInteraction
{
    fn from(
        interaction: crate::query::substance_query::SubstanceQuerySubstancesDangerousInteractions,
    ) -> DangerousInteraction {
        DangerousInteraction {
            name: interaction.name.unwrap_or_default(),
        }
    }
}

impl From<crate::query::substance_query::SubstanceQuerySubstancesUnsafeInteractions>
    for UnsafeInteraction
{
    fn from(
        interaction: crate::query::substance_query::SubstanceQuerySubstancesUnsafeInteractions,
    ) -> UnsafeInteraction {
        UnsafeInteraction {
            name: interaction.name.unwrap_or_default(),
        }
    }
}

impl From<crate::query::substance_query::SubstanceQuerySubstancesUncertainInteractions>
    for UncertainInteraction
{
    fn from(
        interaction: crate::query::substance_query::SubstanceQuerySubstancesUncertainInteractions,
    ) -> UncertainInteraction {
        UncertainInteraction {
            name: interaction.name.unwrap_or_default(),
        }
    }
}

impl From<crate::query::substance_query::SubstanceQuerySubstancesRoas> for RouteOfAdministration {
    fn from(roa: crate::query::substance_query::SubstanceQuerySubstancesRoas) -> Self {
        RouteOfAdministration {
            ty: roa.name.map(|i| i.into()).unwrap_or_default(),
            dose_metadata: roa.dose.map(|i| i.into()).unwrap_or_default(),
            duration: roa.duration.map(|i| i.into()).unwrap_or_default(),
        }
    }
}

impl From<crate::query::substance_query::SubstanceQuerySubstancesRoasDose> for DoseMetadata {
    fn from(dosage: crate::query::substance_query::SubstanceQuerySubstancesRoasDose) -> DoseMetadata {
        DoseMetadata {
            units: dosage.units.unwrap_or_default().into(),
            threshold: dosage.threshold,
            heavy: dosage.heavy,
            common: dosage
                .common
                .map(|i| i.min.unwrap_or_default()..i.max.unwrap_or_default()),
            light: dosage
                .light
                .map(|i| i.min.unwrap_or_default()..i.max.unwrap_or_default()),
            strong: dosage
                .strong
                .map(|i| i.min.unwrap_or_default()..i.max.unwrap_or_default()),
        }
    }
}

impl From<crate::query::substance_query::SubstanceQuerySubstancesRoasDuration> for Duration {
    fn from(
        duration: crate::query::substance_query::SubstanceQuerySubstancesRoasDuration,
    ) -> Duration {
        Duration {
            afterglow: duration.afterglow.map(|i| {
                let units = i.units.unwrap_or_default().into();
                let start = i.min.unwrap_or_default();
                let end = i.max.unwrap_or_default();
                let midpoint = (start + end) / 2.0;

                let duration = match units {
                    TimeUnits::Minutes => std::time::Duration::from_secs_f64(end * 60.0),
                    TimeUnits::Hours => std::time::Duration::from_secs_f64(end * 3600.0),
                    _ => unimplemented!(),
                };

                DoseTimeRange {
                    duration,
                    units,
                    start,
                    midpoint,
                    end,
                }
            }),
            comeup: duration.comeup.map(|i| {
                let units = i.units.unwrap_or_default().into();
                let start = i.min.unwrap_or_default();
                let end = i.max.unwrap_or_default();
                let midpoint = (start + end) / 2.0;

                let duration = match units {
                    TimeUnits::Minutes => std::time::Duration::from_secs_f64(end * 60.0),
                    TimeUnits::Hours => std::time::Duration::from_secs_f64(end * 3600.0),
                    _ => unimplemented!(),
                };

                DoseTimeRange {
                    duration,
                    units,
                    start,
                    midpoint,
                    end,
                }
            }),
            duration: duration.duration.map(|i| {
                let units = i.units.unwrap_or_default().into();
                let start = i.min.unwrap_or_default();
                let end = i.max.unwrap_or_default();
                let midpoint = (start + end) / 2.0;

                let duration = match units {
                    TimeUnits::Minutes => std::time::Duration::from_secs_f64(end * 60.0),
                    TimeUnits::Hours => std::time::Duration::from_secs_f64(end * 3600.0),
                    _ => unimplemented!(),
                };

                DoseTimeRange {
                    duration,
                    units,
                    start,
                    midpoint,
                    end,
                }
            }),
            offset: duration.offset.map(|i| {
                let units = i.units.unwrap_or_default().into();
                let start = i.min.unwrap_or_default();
                let end = i.max.unwrap_or_default();
                let midpoint = (start + end) / 2.0;

                let duration = match units {
                    TimeUnits::Minutes => std::time::Duration::from_secs_f64(end * 60.0),
                    TimeUnits::Hours => std::time::Duration::from_secs_f64(end * 3600.0),
                    _ => unimplemented!(),
                };

                DoseTimeRange {
                    duration,
                    units,
                    start,
                    midpoint,
                    end,
                }
            }),
            onset: duration.onset.map(|i| {
                let units = i.units.unwrap_or_default().into();
                let start = i.min.unwrap_or_default();
                let end = i.max.unwrap_or_default();
                let midpoint = (start + end) / 2.0;

                let duration = match units {
                    TimeUnits::Minutes => std::time::Duration::from_secs_f64(end * 60.0),
                    TimeUnits::Hours => std::time::Duration::from_secs_f64(end * 3600.0),
                    _ => unimplemented!(),
                };

                DoseTimeRange {
                    duration,
                    units,
                    start,
                    midpoint,
                    end,
                }
            }),
            peak: duration.peak.map(|i| {
                let units = i.units.unwrap_or_default().into();
                let start = i.min.unwrap_or_default();
                let end = i.max.unwrap_or_default();
                let midpoint = (start + end) / 2.0;

                let duration = match units {
                    TimeUnits::Minutes => std::time::Duration::from_secs_f64(end * 60.0),
                    TimeUnits::Hours => std::time::Duration::from_secs_f64(end * 3600.0),
                    _ => unimplemented!(),
                };

                DoseTimeRange {
                    duration,
                    units,
                    start,
                    midpoint,
                    end,
                }
            }),
            total: duration.total.map(|i| {
                let units = i.units.unwrap_or_default().into();
                let start = i.min.unwrap_or_default();
                let end = i.max.unwrap_or_default();
                let midpoint = (start + end) / 2.0;

                let duration = match units {
                    TimeUnits::Minutes => std::time::Duration::from_secs_f64(end * 60.0),
                    TimeUnits::Hours => std::time::Duration::from_secs_f64(end * 3600.0),
                    _ => unimplemented!(),
                };

                DoseTimeRange {
                    duration,
                    units,
                    start,
                    midpoint,
                    end,
                }
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_query() {
        dbg!(SubstanceQuery::substance_data("LSD")
            .await
            .unwrap());
    }
}
