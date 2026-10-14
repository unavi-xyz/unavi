//! What each trust rung is granted.

use std::sync::Arc;

use bevy::ecs::resource::Resource;

use crate::{
    permissions::{
        HostApi,
        Permissions,
    },
    quota::limits::Limits,
    trust::Trust,
};

/// Multiple of [`Limits::peer`] granted to the local user's own content.
const MYSELF_SHARE: f64 = 4.0;

/// What one rung may call and consume.
#[derive(Clone)]
pub struct Grant {
    pub permissions: Permissions,
    pub limits:      Limits,
    /// Whether a peer at this rung that leaves has its live state adopted:
    /// its session opinions copied into the viewer's own, and the documents
    /// it authored kept loaded.
    pub adopts:      bool,
}

/// The only mapping from a trust rung to its grant.
#[derive(Resource, Clone)]
pub struct Rules(Arc<[Grant; 5]>);

impl Default for Rules {
    /// The rungs gate only what is uncommon and consequential. Reading the
    /// scene, speaking on channels, spawning documents and opening portals are
    /// bounded by quota, so they sit at the floor.
    fn default() -> Self {
        let anonymous = Permissions::of(&[
            HostApi::Event,
            HostApi::Input,
            HostApi::Peer,
            HostApi::Scene,
        ]);
        let guest = anonymous.union(Permissions::of(&[HostApi::CreateDocument, HostApi::Portal]));
        // Identity is the durable handle the trust model is keyed to, and the
        // agent pose is motion capture of a real person. Commit makes an
        // opinion durable, so content standing in its author's room must not
        // hold it. The shell and its tools are authored at this rung.
        let trusted = guest.union(Permissions::of(&[
            HostApi::Commit,
            HostApi::Identity,
            HostApi::LocalAgent,
        ]));
        // Global input listening, the physics solver and cross-space reach.
        let myself = trusted.union(Permissions::of(&[
            HostApi::InputContext,
            HostApi::Physics,
            HostApi::Storage,
            HostApi::Travel,
        ]));

        let peer = Limits::peer();
        Self(Arc::new([
            Grant {
                permissions: Permissions::none(),
                limits:      peer.scaled(0.0),
                adopts:      false,
            },
            Grant {
                permissions: anonymous,
                limits:      peer.scaled(0.05),
                adopts:      false,
            },
            Grant {
                permissions: guest,
                limits:      peer.scaled(0.25),
                adopts:      true,
            },
            Grant {
                permissions: trusted,
                limits:      peer.scaled(1.0),
                adopts:      true,
            },
            Grant {
                permissions: myself,
                limits:      peer.scaled(MYSELF_SHARE),
                adopts:      true,
            },
        ]))
    }
}

/// Where `trust`'s grant sits in [`Rules`].
const fn slot(trust: Trust) -> usize {
    match trust {
        Trust::Blocked => 0,
        Trust::Anonymous => 1,
        Trust::Guest => 2,
        Trust::Trusted => 3,
        Trust::Myself => 4,
    }
}

impl Rules {
    #[must_use]
    pub fn grant(&self, trust: Trust) -> &Grant {
        &self.0[slot(trust)]
    }

    #[must_use]
    pub fn permissions(&self, trust: Trust) -> Permissions {
        self.grant(trust).permissions
    }

    #[must_use]
    pub fn limits(&self, trust: Trust) -> Limits {
        self.grant(trust).limits.clone()
    }

    /// Replaces `trust`'s grant. A blocked author keeps an empty grant, so a
    /// grant for [`Trust::Blocked`] is ignored.
    #[must_use]
    pub fn with(mut self, trust: Trust, grant: Grant) -> Self {
        if trust != Trust::Blocked {
            Arc::make_mut(&mut self.0)[slot(trust)] = grant;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quota::{
        Flow,
        Quota,
        Reservation,
        Stock,
    };

    const ALL: [HostApi; 13] = [
        HostApi::Commit,
        HostApi::CreateDocument,
        HostApi::Event,
        HostApi::Identity,
        HostApi::Input,
        HostApi::InputContext,
        HostApi::LocalAgent,
        HostApi::Peer,
        HostApi::Physics,
        HostApi::Portal,
        HostApi::Scene,
        HostApi::Storage,
        HostApi::Travel,
    ];

    const LADDER: [Trust; 5] = [
        Trust::Blocked,
        Trust::Anonymous,
        Trust::Guest,
        Trust::Trusted,
        Trust::Myself,
    ];

    fn prims(rules: &Rules, trust: Trust) -> u64 {
        rules.limits(trust).stock[&Stock::Prims]
    }

    #[test]
    fn a_rung_holds_everything_the_rung_below_it_does() {
        let rules = Rules::default();
        for pair in LADDER.windows(2) {
            let lower = rules.permissions(pair[0]);
            let upper = rules.permissions(pair[1]);
            assert_eq!(
                lower.union(upper),
                upper,
                "{:?} must not drop anything {:?} holds",
                pair[1],
                pair[0]
            );
        }
    }

    #[test]
    fn a_blocked_author_reaches_nothing() {
        let blocked = Rules::default().permissions(Trust::Blocked);
        for name in ALL {
            assert!(blocked.require(name).is_err());
        }
    }

    #[test]
    fn an_unproven_author_cannot_spawn_or_send_anyone_away() {
        let anonymous = Rules::default().permissions(Trust::Anonymous);
        assert!(anonymous.require(HostApi::CreateDocument).is_err());
        assert!(anonymous.require(HostApi::Portal).is_err());
        assert!(anonymous.require(HostApi::Scene).is_ok());
    }

    #[test]
    fn a_first_time_visitors_prop_works_with_no_configuration() {
        let guest = Rules::default().permissions(Trust::Guest);
        for name in [
            HostApi::CreateDocument,
            HostApi::Event,
            HostApi::Input,
            HostApi::Peer,
            HostApi::Portal,
            HostApi::Scene,
        ] {
            assert!(guest.require(name).is_ok(), "a guest needs {name:?}");
        }
    }

    #[test]
    fn a_strangers_document_cannot_read_the_local_user() {
        let guest = Rules::default().permissions(Trust::Guest);
        assert!(
            guest.require(HostApi::Identity).is_err(),
            "a DID is the durable handle the whole trust model is keyed to"
        );
        assert!(
            guest.require(HostApi::LocalAgent).is_err(),
            "the agent pose is continuous motion capture of a real person"
        );
    }

    #[test]
    fn only_the_local_users_own_content_reaches_the_shell_apis() {
        let rules = Rules::default();
        for name in [
            HostApi::InputContext,
            HostApi::Physics,
            HostApi::Storage,
            HostApi::Travel,
        ] {
            assert!(rules.permissions(Trust::Trusted).require(name).is_err());
            assert!(rules.permissions(Trust::Myself).require(name).is_ok());
        }
    }

    #[test]
    fn the_trust_levels_are_ordered_by_what_they_may_consume() {
        let rules = Rules::default();

        assert!(prims(&rules, Trust::Blocked) < prims(&rules, Trust::Anonymous));
        assert!(prims(&rules, Trust::Anonymous) < prims(&rules, Trust::Guest));
        assert!(prims(&rules, Trust::Guest) < prims(&rules, Trust::Trusted));
        assert!(prims(&rules, Trust::Trusted) <= prims(&rules, Trust::Myself));
    }

    #[test]
    fn the_local_users_budget_is_finite() {
        let rules = Rules::default();
        let quota = Quota::new(rules.limits(Trust::Myself), None);

        assert!(quota.charge(Stock::Prims, u64::MAX).is_err());
        assert_eq!(
            quota.try_take(Flow::CreatePrim, u32::MAX),
            Reservation::Never
        );
    }

    #[test]
    fn a_blocked_peer_gets_nothing_and_is_refused_immediately() {
        let quota = Quota::new(Rules::default().limits(Trust::Blocked), None);

        assert_eq!(
            quota.try_take(Flow::CreatePrim, 1),
            Reservation::Never,
            "a zero bucket never fills, so waiting on it would be a lie"
        );
        assert!(quota.charge(Stock::Prims, 1).is_err());
    }

    #[test]
    fn a_guest_still_gets_a_workable_budget() {
        let quota = Quota::new(Rules::default().limits(Trust::Guest), None);
        assert_eq!(
            quota.try_take(Flow::CreatePrim, 100),
            Reservation::Ready,
            "a first-time visitor's prop must build without waiting"
        );
    }

    #[test]
    fn an_override_replaces_one_rung_and_leaves_the_rest() {
        let grant = Grant {
            permissions: Permissions::of(&[HostApi::Event]),
            limits:      Limits::default(),
            adopts:      true,
        };
        let rules = Rules::default().with(Trust::Guest, grant);

        assert!(
            rules
                .permissions(Trust::Guest)
                .require(HostApi::Portal)
                .is_err()
        );
        assert!(
            rules
                .permissions(Trust::Anonymous)
                .require(HostApi::Scene)
                .is_ok()
        );
    }

    #[test]
    fn an_override_cannot_grant_anything_to_a_blocked_author() {
        let grant = Grant {
            permissions: Permissions::of(&ALL),
            limits:      Limits::peer(),
            adopts:      true,
        };
        let rules = Rules::default().with(Trust::Blocked, grant);

        assert_eq!(rules.permissions(Trust::Blocked), Permissions::none());
        assert_eq!(rules.limits(Trust::Blocked).stock[&Stock::Prims], 0);
        assert!(!rules.grant(Trust::Blocked).adopts);
    }

    #[test]
    fn a_departing_peer_is_adopted_from_guest_up() {
        let rules = Rules::default();
        let adopted = LADDER.map(|trust| rules.grant(trust).adopts);
        assert_eq!(
            adopted,
            [false, false, true, true, true],
            "an unproven or blocked peer leaves nothing behind"
        );
    }
}
