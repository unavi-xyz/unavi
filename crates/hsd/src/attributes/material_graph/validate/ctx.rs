use super::error::GraphError;
use crate::attributes::material_graph::{
    node::{
        Network,
        Port,
    },
    value::{
        GraphValue,
        ValueKind,
        is_finite,
    },
};

/// What every port rule is resolved against: the network being validated, the
/// graph's public inputs, the index of the node whose ports are being checked,
/// and the output kinds of the nodes before it.
pub(super) struct Ctx<'a> {
    pub(super) network:       Network,
    pub(super) public_inputs: &'a [GraphValue],
    /// A [`Port::Node`] must reference a strictly lower index than this,
    /// which is the entire cycle check this format needs. A terminal sits
    /// "after" every node, so it uses `kinds.len()`.
    pub(super) at:            usize,
    pub(super) kinds:         &'a [ValueKind],
}

impl Ctx<'_> {
    pub(super) fn port_kind(&self, port: Port) -> Result<ValueKind, GraphError> {
        let (network, node) = (self.network, self.at);
        match port {
            Port::Const(value) if !is_finite(value) => {
                Err(GraphError::NonFiniteConst { network, node })
            }
            Port::Const(value) => Ok(value.kind()),
            Port::Input(index) => self
                .public_inputs
                .get(usize::from(index))
                .map(GraphValue::kind)
                .ok_or(GraphError::UnknownInput {
                    network,
                    node,
                    index,
                }),
            Port::Node(target) => {
                if usize::from(target) >= self.at {
                    return Err(GraphError::ForwardReference {
                        network,
                        node,
                        target,
                    });
                }
                Ok(self.kinds[usize::from(target)])
            }
        }
    }

    pub(super) fn require(
        &self,
        name: &'static str,
        port: Port,
        expected: ValueKind,
    ) -> Result<(), GraphError> {
        let found = self.port_kind(port)?;
        if found == expected {
            Ok(())
        } else {
            Err(GraphError::NodeTypeMismatch {
                network: self.network,
                node: self.at,
                port: name,
                expected,
                found,
            })
        }
    }

    /// Both ports carry one kind, taken from `a`.
    pub(super) fn matching(
        &self,
        a: Port,
        b_name: &'static str,
        b: Port,
    ) -> Result<ValueKind, GraphError> {
        let kind = self.port_kind(a)?;
        self.require(b_name, b, kind)?;
        Ok(kind)
    }

    /// [`Ctx::matching`] over more than two ports: `rest` all take `first`'s
    /// kind.
    pub(super) fn all_matching(
        &self,
        first: Port,
        rest: &[(&'static str, Port)],
    ) -> Result<ValueKind, GraphError> {
        let kind = self.port_kind(first)?;
        for (name, port) in rest {
            self.require(name, *port, kind)?;
        }
        Ok(kind)
    }

    /// The arithmetic nodes' rule: either two operands of one kind, or a
    /// vector and a `Float`, which broadcasts across its components. WGSL's
    /// `+ - * /` accept mixed scalar/vector operands natively, so this is
    /// purely a validation rule with no codegen counterpart — the
    /// builtin-backed nodes use [`Ctx::matching`] instead.
    pub(super) fn arithmetic(&self, a: Port, b: Port) -> Result<ValueKind, GraphError> {
        let (a_kind, b_kind) = (self.port_kind(a)?, self.port_kind(b)?);
        match (a_kind, b_kind) {
            (a, b) if a == b => Ok(a),
            (ValueKind::Float, vector) | (vector, ValueKind::Float) => Ok(vector),
            _ => Err(GraphError::NodeTypeMismatch {
                network:  self.network,
                node:     self.at,
                port:     "b",
                expected: a_kind,
                found:    b_kind,
            }),
        }
    }

    /// A port whose meaning is undefined on a scalar.
    pub(super) fn vector_port(
        &self,
        name: &'static str,
        port: Port,
    ) -> Result<ValueKind, GraphError> {
        let found = self.port_kind(port)?;
        if found.is_vector() {
            Ok(found)
        } else {
            Err(GraphError::NotAVector {
                network: self.network,
                node: self.at,
                port: name,
                found,
            })
        }
    }

    /// Assembles a vector out of scalars, for the `Combine` nodes.
    pub(super) fn combine(
        &self,
        ports: &[(&'static str, Port)],
        out: ValueKind,
    ) -> Result<ValueKind, GraphError> {
        for (name, port) in ports {
            self.require(name, *port, ValueKind::Float)?;
        }
        Ok(out)
    }

    pub(super) fn extract(&self, v: Port, channel: u8) -> Result<ValueKind, GraphError> {
        let kind = self.vector_port("v", v)?;
        if channel >= kind.components() {
            return Err(GraphError::ChannelOutOfRange {
                network: self.network,
                node: self.at,
                channel,
                kind,
            });
        }
        Ok(ValueKind::Float)
    }

    pub(super) fn convert(&self, v: Port, to: ValueKind) -> Result<ValueKind, GraphError> {
        let from = self.port_kind(v)?;
        if from.is_vector() && to.is_vector() {
            Ok(to)
        } else {
            Err(GraphError::InvalidConversion {
                network: self.network,
                node: self.at,
                from,
                to,
            })
        }
    }
}
