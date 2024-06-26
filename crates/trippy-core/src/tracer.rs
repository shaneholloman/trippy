use crate::error::TraceResult;
use crate::{
    IcmpExtensionParseMode, MaxInflight, MaxRounds, MultipathStrategy, PacketSize, PayloadPattern,
    PortDirection, PrivilegeMode, Protocol, Sequence, TimeToLive, TraceId, TraceState, TracerError,
    TracerRound, TypeOfService,
};
use std::fmt::Debug;
use std::net::IpAddr;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

/// A traceroute implementation.
///
/// See the [`crate`] documentation for more information.
///
/// Note that this is type cheaply cloneable.
#[derive(Debug, Clone)]
pub struct Tracer {
    inner: Arc<inner::TracerInner>,
}

impl Tracer {
    /// Create a `Tracer`.
    ///
    /// Use the [`crate::Builder`] type to create a [`Tracer`].
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub(crate) fn new(
        interface: Option<String>,
        source_addr: Option<IpAddr>,
        target_addr: IpAddr,
        privilege_mode: PrivilegeMode,
        protocol: Protocol,
        packet_size: PacketSize,
        payload_pattern: PayloadPattern,
        tos: TypeOfService,
        icmp_extension_parse_mode: IcmpExtensionParseMode,
        read_timeout: Duration,
        tcp_connect_timeout: Duration,
        trace_identifier: TraceId,
        max_rounds: Option<MaxRounds>,
        first_ttl: TimeToLive,
        max_ttl: TimeToLive,
        grace_duration: Duration,
        max_inflight: MaxInflight,
        initial_sequence: Sequence,
        multipath_strategy: MultipathStrategy,
        port_direction: PortDirection,
        min_round_duration: Duration,
        max_round_duration: Duration,
        max_samples: usize,
        max_flows: usize,
        drop_privileges: bool,
    ) -> Self {
        Self {
            inner: Arc::new(inner::TracerInner::new(
                interface,
                source_addr,
                target_addr,
                privilege_mode,
                protocol,
                packet_size,
                payload_pattern,
                tos,
                icmp_extension_parse_mode,
                read_timeout,
                tcp_connect_timeout,
                trace_identifier,
                max_rounds,
                first_ttl,
                max_ttl,
                grace_duration,
                max_inflight,
                initial_sequence,
                multipath_strategy,
                port_direction,
                min_round_duration,
                max_round_duration,
                max_samples,
                max_flows,
                drop_privileges,
            )),
        }
    }

    /// Run the [`Tracer`].
    ///
    /// This method will block until either the trace completes all rounds (if
    /// [`crate::Builder::max_rounds`] has been called to set to a non-zero
    /// value) or until the trace fails.
    ///
    /// At the completion of the trace, the state of the tracer can be
    /// retrieved using the [`Tracer::snapshot`] method.
    ///
    /// If you want to run the tracer indefinitely (by not setting
    /// [`crate::Builder::max_rounds`]), you can either clone and run the
    /// tracer on a separate thread by using the [`Tracer::spawn`] method or
    /// by use the [`Tracer::run_with`] method in the current thread to gather
    /// pee round state manually.
    ///
    /// # Example
    ///
    /// The following will run the tracer for a fixed number (3) of rounds and
    /// then retrieve the final state snapshot:
    ///
    /// ```no_run
    /// # fn main() -> anyhow::Result<()> {
    /// # use std::net::IpAddr;
    /// # use std::str::FromStr;
    /// use trippy_core::Builder;
    ///
    /// let addr = IpAddr::from_str("1.1.1.1")?;
    /// let tracer = Builder::new(addr).max_rounds(Some(3)).build()?;
    /// tracer.run()?;
    /// let _state = tracer.snapshot();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`Tracer::run_with`] - Run the tracer with a custom round handler.
    /// - [`Tracer::spawn`] - Spawn the tracer on a new thread without a
    /// custom round handler.
    pub fn run(&self) -> TraceResult<()> {
        self.inner.run()
    }

    /// Run the [`Tracer`] with a custom round handler.
    ///
    /// This method will block until either the trace completes all rounds (if
    /// [`crate::Builder::max_rounds`] has been called to set to a non-zero
    /// value) or until the trace fails.
    ///
    /// At the completion of the trace, the state of the tracer can be
    /// retrieved using the [`Tracer::snapshot`] method.
    ///
    /// This method will additionally call the provided function for each round
    /// that is completed.  This can be useful if you want to gather round state
    /// manually if the tracer is run indefinitely (by not setting
    /// [`crate::Builder::max_rounds`])
    ///
    /// # Example
    ///
    /// The following will run the tracer indefinitely and print the data from
    /// each round of tracing:
    ///
    /// ```no_run
    /// # fn main() -> anyhow::Result<()> {
    /// # use std::net::IpAddr;
    /// # use std::str::FromStr;
    /// use trippy_core::Builder;
    ///
    /// let addr = IpAddr::from_str("1.1.1.1")?;
    /// let tracer = Builder::new(addr).build()?;
    /// tracer.run_with(|round| println!("{:?}", round))?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`Tracer::run`] - Run the tracer without a custom round handler.
    pub fn run_with<F: Fn(&TracerRound<'_>)>(&self, func: F) -> TraceResult<()> {
        self.inner.run_with(func)
    }

    /// Spawn the tracer on a new thread.
    ///
    /// This method will spawn a new thread to run the tracer and immediately
    /// return the [`Tracer`] and a handle to the thread, so it may be joined
    /// with [`JoinHandle::join`].
    ///
    /// If you want to run the tracer indefinitely (by not setting
    /// [`crate::Builder::max_rounds`]) you can use this method to spawn the
    /// tracer on a new thread and return the [`Tracer`] such that a
    /// [`Tracer::snapshot`] of the state can be taken at any time.
    ///
    /// # Example
    ///
    /// The following will spawn a tracer on a new thread and take a snapshot
    /// of the state every 5 seconds:
    ///
    /// ```no_run
    /// # fn main() -> anyhow::Result<()> {
    /// # use std::net::IpAddr;
    /// # use std::str::FromStr;
    /// # use std::thread;
    /// # use std::time::Duration;
    /// use trippy_core::Builder;
    ///
    /// let addr = IpAddr::from_str("1.1.1.1")?;
    /// let (tracer, _) = Builder::new(addr).build()?.spawn()?;
    /// loop {
    ///     thread::sleep(Duration::from_secs(5));
    ///     // get the latest state.
    ///     let _state = tracer.snapshot();
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`Tracer::run`] - Run the tracer on the current thread.
    pub fn spawn(self) -> TraceResult<(Self, JoinHandle<TraceResult<()>>)> {
        let tracer = self.clone();
        let handle = thread::Builder::new()
            .name(format!("tracer-{}", self.trace_identifier().0))
            .spawn(move || tracer.run())
            .map_err(|err| TracerError::Other(err.to_string()))?;
        Ok((self, handle))
    }

    /// Spawn the tracer with a custom round handler on a new thread.
    ///
    /// This method will spawn a new thread to run the tracer with a custom
    /// round handler and immediately return the [`Tracer`] and a handle to the
    /// thread, so it may be joined with [`JoinHandle::join`].
    ///
    /// # Example
    ///
    /// The following will spawn a tracer on a new thread with a custom round
    /// handler to print the data from each round of tracing and also take a
    /// snapshot of the state every 5 seconds until the tracer completes all
    /// rounds:
    ///
    /// ```no_run
    /// # fn main() -> anyhow::Result<()> {
    /// # use std::net::IpAddr;
    /// # use std::str::FromStr;
    /// # use std::thread;
    /// # use std::time::Duration;
    /// use trippy_core::Builder;
    ///
    /// let addr = IpAddr::from_str("1.1.1.1")?;
    /// let (tracer, handle) = Builder::new(addr)
    ///     .max_rounds(Some(3))
    ///     .build()?
    ///     .spawn_with(|round| println!("{:?}", round))?;
    /// for i in 0..3 {
    ///     thread::sleep(Duration::from_secs(5));
    ///     // get the latest state.
    ///     let _state = tracer.snapshot();
    /// }
    /// handle.join().unwrap()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`Tracer::spawn`] - Spawn the tracer on a new thread without a
    /// custom round handler.
    pub fn spawn_with<F: Fn(&TracerRound<'_>) + Send + 'static>(
        self,
        func: F,
    ) -> TraceResult<(Self, JoinHandle<TraceResult<()>>)> {
        let tracer = self.clone();
        let handle = thread::Builder::new()
            .name(format!("tracer-{}", self.trace_identifier().0))
            .spawn(move || tracer.run_with(func))
            .map_err(|err| TracerError::Other(err.to_string()))?;
        Ok((self, handle))
    }

    /// Take a snapshot of the tracer state.
    #[must_use]
    pub fn snapshot(&self) -> TraceState {
        self.inner.snapshot()
    }

    /// Clear the tracer state.
    pub fn clear(&self) {
        self.inner.clear();
    }

    #[must_use]
    pub fn max_flows(&self) -> usize {
        self.inner.max_flows()
    }

    #[must_use]
    pub fn max_samples(&self) -> usize {
        self.inner.max_samples()
    }

    #[must_use]
    pub fn privilege_mode(&self) -> PrivilegeMode {
        self.inner.privilege_mode()
    }

    #[must_use]
    pub fn protocol(&self) -> Protocol {
        self.inner.protocol()
    }

    #[must_use]
    pub fn interface(&self) -> Option<&str> {
        self.inner.interface()
    }

    #[must_use]
    pub fn source_addr(&self) -> Option<IpAddr> {
        self.inner.source_addr()
    }

    #[must_use]
    pub fn target_addr(&self) -> IpAddr {
        self.inner.target_addr()
    }

    #[must_use]
    pub fn packet_size(&self) -> PacketSize {
        self.inner.packet_size()
    }

    #[must_use]
    pub fn payload_pattern(&self) -> PayloadPattern {
        self.inner.payload_pattern()
    }

    #[must_use]
    pub fn initial_sequence(&self) -> Sequence {
        self.inner.initial_sequence()
    }

    #[must_use]
    pub fn tos(&self) -> TypeOfService {
        self.inner.tos()
    }

    #[must_use]
    pub fn icmp_extension_parse_mode(&self) -> IcmpExtensionParseMode {
        self.inner.icmp_extension_parse_mode()
    }

    #[must_use]
    pub fn read_timeout(&self) -> Duration {
        self.inner.read_timeout()
    }

    #[must_use]
    pub fn tcp_connect_timeout(&self) -> Duration {
        self.inner.tcp_connect_timeout()
    }

    #[must_use]
    pub fn trace_identifier(&self) -> TraceId {
        self.inner.trace_identifier()
    }

    #[must_use]
    pub fn max_rounds(&self) -> Option<MaxRounds> {
        self.inner.max_rounds()
    }

    #[must_use]
    pub fn first_ttl(&self) -> TimeToLive {
        self.inner.first_ttl()
    }

    #[must_use]
    pub fn max_ttl(&self) -> TimeToLive {
        self.inner.max_ttl()
    }

    #[must_use]
    pub fn grace_duration(&self) -> Duration {
        self.inner.grace_duration()
    }

    #[must_use]
    pub fn max_inflight(&self) -> MaxInflight {
        self.inner.max_inflight()
    }

    #[must_use]
    pub fn multipath_strategy(&self) -> MultipathStrategy {
        self.inner.multipath_strategy()
    }

    #[must_use]
    pub fn port_direction(&self) -> PortDirection {
        self.inner.port_direction()
    }

    #[must_use]
    pub fn min_round_duration(&self) -> Duration {
        self.inner.min_round_duration()
    }

    #[must_use]
    pub fn max_round_duration(&self) -> Duration {
        self.inner.max_round_duration()
    }
}

mod inner {
    use crate::config::{ChannelConfig, StateConfig, StrategyConfig};
    use crate::error::TraceResult;
    use crate::net::{PlatformImpl, SocketImpl};
    use crate::{
        IcmpExtensionParseMode, MaxInflight, MaxRounds, MultipathStrategy, PacketSize,
        PayloadPattern, PortDirection, PrivilegeMode, Protocol, Sequence, SourceAddr, TimeToLive,
        TraceId, TraceState, TracerChannel, TracerError, TracerRound, TracerStrategy,
        TypeOfService,
    };
    use parking_lot::RwLock;
    use std::fmt::Debug;
    use std::net::IpAddr;
    use std::sync::OnceLock;
    use std::time::Duration;
    use tracing::instrument;
    use trippy_privilege::Privilege;

    #[derive(Debug)]
    pub(super) struct TracerInner {
        source_addr: Option<IpAddr>,
        interface: Option<String>,
        target_addr: IpAddr,
        privilege_mode: PrivilegeMode,
        protocol: Protocol,
        packet_size: PacketSize,
        payload_pattern: PayloadPattern,
        tos: TypeOfService,
        icmp_extension_parse_mode: IcmpExtensionParseMode,
        read_timeout: Duration,
        tcp_connect_timeout: Duration,
        trace_identifier: TraceId,
        max_rounds: Option<MaxRounds>,
        first_ttl: TimeToLive,
        max_ttl: TimeToLive,
        grace_duration: Duration,
        max_inflight: MaxInflight,
        initial_sequence: Sequence,
        multipath_strategy: MultipathStrategy,
        port_direction: PortDirection,
        min_round_duration: Duration,
        max_round_duration: Duration,
        max_samples: usize,
        max_flows: usize,
        drop_privileges: bool,
        state: RwLock<TraceState>,
        src: OnceLock<IpAddr>,
    }

    impl TracerInner {
        #[allow(clippy::too_many_arguments)]
        pub(super) fn new(
            interface: Option<String>,
            source_addr: Option<IpAddr>,
            target_addr: IpAddr,
            privilege_mode: PrivilegeMode,
            protocol: Protocol,
            packet_size: PacketSize,
            payload_pattern: PayloadPattern,
            tos: TypeOfService,
            icmp_extension_parse_mode: IcmpExtensionParseMode,
            read_timeout: Duration,
            tcp_connect_timeout: Duration,
            trace_identifier: TraceId,
            max_rounds: Option<MaxRounds>,
            first_ttl: TimeToLive,
            max_ttl: TimeToLive,
            grace_duration: Duration,
            max_inflight: MaxInflight,
            initial_sequence: Sequence,
            multipath_strategy: MultipathStrategy,
            port_direction: PortDirection,
            min_round_duration: Duration,
            max_round_duration: Duration,
            max_samples: usize,
            max_flows: usize,
            drop_privileges: bool,
        ) -> Self {
            Self {
                source_addr,
                interface,
                target_addr,
                privilege_mode,
                protocol,
                packet_size,
                payload_pattern,
                tos,
                icmp_extension_parse_mode,
                read_timeout,
                tcp_connect_timeout,
                trace_identifier,
                max_rounds,
                first_ttl,
                max_ttl,
                grace_duration,
                max_inflight,
                initial_sequence,
                multipath_strategy,
                port_direction,
                min_round_duration,
                max_round_duration,
                max_samples,
                max_flows,
                drop_privileges,
                state: RwLock::new(TraceState::new(Self::make_state_config(
                    max_flows,
                    max_samples,
                ))),
                src: OnceLock::new(),
            }
        }

        #[instrument(skip_all)]
        pub(super) fn run(&self) -> TraceResult<()> {
            self.run_internal(|_| ())
                .map_err(|err| self.handle_error(err))
        }

        #[instrument(skip_all)]
        pub(super) fn run_with<F: Fn(&TracerRound<'_>)>(&self, func: F) -> TraceResult<()> {
            self.run_internal(func)
                .map_err(|err| self.handle_error(err))
        }

        pub(super) fn snapshot(&self) -> TraceState {
            self.state.read().clone()
        }

        pub(super) fn clear(&self) {
            *self.state.write() =
                TraceState::new(Self::make_state_config(self.max_flows, self.max_samples));
        }

        pub(super) fn max_flows(&self) -> usize {
            self.max_flows
        }

        pub(super) fn max_samples(&self) -> usize {
            self.max_samples
        }

        pub(super) fn privilege_mode(&self) -> PrivilegeMode {
            self.privilege_mode
        }

        pub(super) fn protocol(&self) -> Protocol {
            self.protocol
        }

        pub(super) fn interface(&self) -> Option<&str> {
            self.interface.as_deref()
        }

        pub(super) fn source_addr(&self) -> Option<IpAddr> {
            self.src.get().copied()
        }

        pub(super) fn target_addr(&self) -> IpAddr {
            self.target_addr
        }

        pub(super) fn packet_size(&self) -> PacketSize {
            self.packet_size
        }

        pub(super) fn payload_pattern(&self) -> PayloadPattern {
            self.payload_pattern
        }

        pub(super) fn initial_sequence(&self) -> Sequence {
            self.initial_sequence
        }

        pub(super) fn tos(&self) -> TypeOfService {
            self.tos
        }

        pub(super) fn icmp_extension_parse_mode(&self) -> IcmpExtensionParseMode {
            self.icmp_extension_parse_mode
        }

        pub(super) fn read_timeout(&self) -> Duration {
            self.read_timeout
        }

        pub(super) fn tcp_connect_timeout(&self) -> Duration {
            self.tcp_connect_timeout
        }

        pub(super) fn trace_identifier(&self) -> TraceId {
            self.trace_identifier
        }

        pub(super) fn max_rounds(&self) -> Option<MaxRounds> {
            self.max_rounds
        }

        pub(super) fn first_ttl(&self) -> TimeToLive {
            self.first_ttl
        }

        pub(super) fn max_ttl(&self) -> TimeToLive {
            self.max_ttl
        }

        pub(super) fn grace_duration(&self) -> Duration {
            self.grace_duration
        }

        pub(super) fn max_inflight(&self) -> MaxInflight {
            self.max_inflight
        }

        pub(super) fn multipath_strategy(&self) -> MultipathStrategy {
            self.multipath_strategy
        }

        pub(super) fn port_direction(&self) -> PortDirection {
            self.port_direction
        }

        pub(super) fn min_round_duration(&self) -> Duration {
            self.min_round_duration
        }

        pub(super) fn max_round_duration(&self) -> Duration {
            self.max_round_duration
        }

        #[instrument(skip_all)]
        fn run_internal<F: Fn(&TracerRound<'_>)>(&self, func: F) -> TraceResult<()> {
            // if we are given a source address, validate it otherwise
            // discover it based on the target address and interface.
            let source_addr = match self.source_addr {
                None => SourceAddr::discover::<SocketImpl, PlatformImpl>(
                    self.target_addr,
                    self.port_direction,
                    self.interface.as_deref(),
                )?,
                Some(addr) => SourceAddr::validate::<SocketImpl>(addr)?,
            };
            self.src
                .set(source_addr)
                .map_err(|_| TracerError::Other(String::from("failed to set source_addr")))?;
            let channel_config = self.make_channel_config(source_addr);
            let channel = TracerChannel::<SocketImpl>::connect(&channel_config)?;
            if self.drop_privileges {
                Privilege::drop_privileges()?;
            }
            let strategy_config = self.make_strategy_config();
            let strategy = TracerStrategy::new(&strategy_config, |round| {
                self.handler(round);
                func(round);
            });
            strategy.run(channel)?;
            Ok(())
        }

        fn handler(&self, round: &TracerRound<'_>) {
            self.state.write().update_from_round(round);
        }

        fn handle_error(&self, err: TracerError) -> TracerError {
            self.state.write().set_error(Some(err.to_string()));
            err
        }

        fn make_state_config(max_flows: usize, max_samples: usize) -> StateConfig {
            StateConfig {
                max_samples,
                max_flows,
            }
        }

        fn make_channel_config(&self, source_addr: IpAddr) -> ChannelConfig {
            ChannelConfig {
                privilege_mode: self.privilege_mode,
                protocol: self.protocol,
                source_addr,
                target_addr: self.target_addr,
                packet_size: self.packet_size,
                payload_pattern: self.payload_pattern,
                initial_sequence: self.initial_sequence,
                tos: self.tos,
                icmp_extension_parse_mode: self.icmp_extension_parse_mode,
                read_timeout: self.read_timeout,
                tcp_connect_timeout: self.tcp_connect_timeout,
            }
        }

        fn make_strategy_config(&self) -> StrategyConfig {
            StrategyConfig {
                target_addr: self.target_addr,
                protocol: self.protocol,
                trace_identifier: self.trace_identifier,
                max_rounds: self.max_rounds,
                first_ttl: self.first_ttl,
                max_ttl: self.max_ttl,
                grace_duration: self.grace_duration,
                max_inflight: self.max_inflight,
                initial_sequence: self.initial_sequence,
                multipath_strategy: self.multipath_strategy,
                port_direction: self.port_direction,
                min_round_duration: self.min_round_duration,
                max_round_duration: self.max_round_duration,
            }
        }
    }
}
