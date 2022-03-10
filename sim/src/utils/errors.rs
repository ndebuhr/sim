use thiserror::Error;

/// `SimulationError` enumerates all possible errors returned by sim
#[derive(Error, Debug)]
pub enum SimulationError {
    /// Represents an invalid model configuration encountered during simulation
    #[error("An invalid model configuration was encountered during simulation")]
    InvalidModelConfiguration,

    /// Represents an operation requested on a model that does not exist
    #[error("A specified model cannot be found in the simulation")]
    ModelNotFound,

    /// Represents an operation requested on a model port that does not exist
    #[error("A specified model port cannot be found in the simulation")]
    PortNotFound,

    /// Represents a failed clone operation on a model
    #[error("A model failed to clone during simulation")]
    ModelCloneError,

    /// Represents an invalid model state
    #[error("An invalid model state was encountered")]
    InvalidModelState,

    /// Represents an invalid state of event scheduling
    #[error("An invalid state was encountered, with respect to event scheduling")]
    EventSchedulingError,

    /// Represents an invalid inter-model message encountered
    #[error("An invalid inter-model message was encountered")]
    InvalidMessage,

    /// Represents a failed serialization operation
    #[error("Failed to serialize a simulation model")]
    SerializationError,

    /// Represents an empty polynomial configuration used in a simulation
    #[error("A polynomial was configured in a simulation, but the coefficients are empty")]
    EmptyPolynomial,

    /// Represents an internal logic error, where prerequisite calculations were not executed
    #[error("An internal logic error occured, where prerequisite calculations were not executed")]
    PrerequisiteCalcError,

    /// Represents a failed conversion to num-traits Float
    #[error("Failed to convert to a Float value")]
    FloatConvError,

    /// Represents a message unexpectedly lost/dropped/stuck during simulation execution
    #[error("A message was unexpectedly lost, dropped, or stuck during simulation execution")]
    DroppedMessageError,

    /// Transparent serde_json errors
    #[error(transparent)]
    JSONError(#[from] serde_json::error::Error),

    /// Transparent Beta distribution errors
    #[error(transparent)]
    BetaError(#[from] rand_distr::BetaError),

    /// Transparent Exponential distribution errors
    #[error(transparent)]
    ExpError(#[from] rand_distr::ExpError),

    /// Transparent Gamma distribution errors
    #[error(transparent)]
    GammaError(#[from] rand_distr::GammaError),

    /// Transparent Normal distribution errors
    #[error(transparent)]
    NormalError(#[from] rand_distr::NormalError),

    /// Transparent Triangular distribution errors
    #[error(transparent)]
    TriangularError(#[from] rand_distr::TriangularError),

    /// Transparent Weibull distribution errors
    #[error(transparent)]
    WeibullError(#[from] rand_distr::WeibullError),

    /// Transparent Bernoulli distribution errors
    #[error(transparent)]
    BernoulliError(#[from] rand_distr::BernoulliError),

    /// Transparent Geometric distribution errors
    #[error(transparent)]
    GeoError(#[from] rand_distr::GeoError),

    /// Transparent Poisson distribution errors
    #[error(transparent)]
    PoissonError(#[from] rand_distr::PoissonError),

    /// Transparent Weighted Index distribution errors
    #[error(transparent)]
    WeightedError(#[from] rand_distr::WeightedError),
}
