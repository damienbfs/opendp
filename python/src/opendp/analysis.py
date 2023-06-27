from typing import Any, Callable, List, Optional, Tuple, Union
import importlib
from inspect import signature
from functools import partial
from opendp.combinators import (
    make_fix_delta,
    make_pureDP_to_fixed_approxDP,
    make_pureDP_to_zCDP,
    make_sequential_composition,
    make_zCDP_to_approxDP,
)
from opendp.domains import atom_domain
from opendp.measurements import make_base_gaussian, make_base_laplace, make_gaussian
from opendp.measures import (
    fixed_smoothed_max_divergence,
    max_divergence,
    zero_concentrated_divergence,
)
from opendp.metrics import (
    absolute_distance,
    change_one_distance,
    hamming_distance,
    insert_delete_distance,
    l1_distance,
    l2_distance,
    symmetric_distance,
)

from opendp.mod import (
    Domain,
    Measurement,
    Metric,
    Queryable,
    Transformation,
    Measure,
    binary_search,
    binary_search_param,
    domain_of,
)
from opendp.typing import RuntimeType

# a dictionary of "constructor name" -> (constructor_function, is_partial)
# "constructor name" is the name of the constructor without the "make_" prefix
# constructor_function is the partial version if is_partial is True
constructors = {}
for module_name in ["transformations", "measurements"]:
    module = importlib.import_module(f"opendp.{module_name}")
    for name in module.__all__:
        if not name.startswith("make_"):
            continue
        partial_name = "then_" + name[5:]
        make_func = getattr(module, name)

        is_partial = partial_name in module.__all__
        constructor = getattr(module, partial_name if is_partial else name)

        constructors[name[5:]] = constructor, is_partial


def loss_of(*, epsilon=None, delta=None, rho=None, U=None) -> Tuple[Measure, float]:
    """Constructs a privacy loss, consisting of a privacy measure and a privacy loss parameter.

    :param U: The type of the privacy parameter.

    >>> from opendp.analysis import loss_of
    >>> measure, distance = loss_of(epsilon=1.0)
    >>> measure, distance = loss_of(epsilon=1.0, delta=1e-9)
    >>> measure, distance = loss_of(rho=1.0)
    """
    if epsilon is None and rho is None:
        raise ValueError("Either epsilon or rho must be specified.")

    if rho:
        U = RuntimeType.parse_or_infer(U, rho)
        return zero_concentrated_divergence(T=U), rho
    if delta is None:
        U = RuntimeType.parse_or_infer(U, epsilon)
        return max_divergence(T=U), epsilon
    else:
        U = RuntimeType.parse_or_infer(U, epsilon)
        return fixed_smoothed_max_divergence(T=U), (epsilon, delta)


def unit_of(
    *,
    contributions=None,
    changes=None,
    absolute=None,
    l1=None,
    l2=None,
    ordered=False,
    U=None,
) -> Tuple[Metric, float]:
    """Constructs a unit of privacy, consisting of a metric and a dataset distance.

    :param ordered: Set to true to use InsertDeleteDistance instead of SymmetricDistance, or HammingDistance instead of ChangeOneDistance.
    :param U: The type of the dataset distance."""

    def _is_distance(p, v):
        return p not in ["ordered", "U", "_is_distance"] and v is not None

    if sum(1 for p, v in locals().items() if _is_distance(p, v)) != 1:
        raise ValueError("Must specify exactly one distance.")

    if contributions is not None:
        metric = insert_delete_distance() if ordered else symmetric_distance()
        return metric, contributions
    if changes is not None:
        metric = hamming_distance() if ordered else change_one_distance()
        return metric, changes
    if absolute is not None:
        metric = absolute_distance(T=RuntimeType.parse_or_infer(U, absolute))
        return metric, absolute
    if l1 is not None:
        metric = l1_distance(T=RuntimeType.parse_or_infer(U, l1))
        return metric, l1
    if l2 is not None:
        metric = l2_distance(T=RuntimeType.parse_or_infer(U, l2))
        return metric, l2


class Analysis(object):
    """An Analysis coordinates queries to an instance of a privacy `accountant`."""

    accountant: Measurement  # union Odometer once merged
    """The accountant is the measurement used to spawn the queryable.
    It contains information about the queryable, 
    such as the input domain, input metric, and output measure expected of measurement queries sent to the queryable."""
    queryable: Queryable
    """The queryable executes the queries and tracks the privacy expenditure."""

    def __init__(
        self,
        accountant: Measurement,
        queryable: Queryable,
        d_in,
        d_mids=None,
        d_out=None,
    ):
        """Initializes the analysis with the given accountant and queryable.

        It is recommended to use the `sequential_composition` constructor instead of this one.

        :param d_in: An upper bound on the distance between adjacent datasets.
        :param d_mids: A sequence of privacy losses for each query to be sent to the queryable. Used for compositors.
        :param d_out: An upper bound on the overall privacy loss. Used for filters."""
        self.accountant = accountant
        self.queryable = queryable
        self.d_in = d_in
        self.d_mids = d_mids
        self.d_out = d_out

    @staticmethod
    def sequential_composition(
        data: Any,
        privacy_unit: Tuple[Metric, float],
        privacy_loss: Tuple[Measure, Any],
        split_evenly_over: Optional[int] = None,
        split_by_weights: Optional[List[float]] = None,
        domain: Optional[Domain] = None,
    ) -> "Analysis":
        """Constructs a new analysis containing a sequential compositor with the given weights.

        If the domain is not specified, it will be inferred from the data.
        This makes the assumption that the structure of the data is public information.

        The weights may be a list of numerics, corresponding to how `privacy_loss` should be distributed to each query.
        Alternatively, pass a single integer to distribute the loss evenly.

        :param data: The data to be analyzed.
        :param privacy_unit: The privacy unit of the analysis.
        :param privacy_loss: The privacy loss of the analysis.
        :param weights: How to distribute `privacy_loss` among the queries.
        :param domain: The domain of the data."""
        if domain is None:
            domain = domain_of(data, infer=True)

        accountant, d_mids = _sequential_composition_by_weights(
            domain, privacy_unit, privacy_loss, split_evenly_over, split_by_weights
        )

        return Analysis(
            accountant=accountant,
            queryable=accountant(data),
            d_in=privacy_unit[1],
            d_mids=d_mids,
        )

    def __call__(self, query: Union["Query", Measurement]):
        """Executes the given query on the analysis."""
        if isinstance(query, Query):
            query = query.resolve()
        answer = self.queryable(query)
        if self.d_mids is not None:
            self.d_mids.pop(0)
        return answer

    def query(self, **kwargs) -> "Query":
        """Starts a new Query to be executed in this analysis.

        If the analysis has been constructed with a sequence of privacy losses,
        the next loss will be used. Otherwise, the loss will be computed from the kwargs.

        :param kwargs: The privacy loss to use for the query. Passed directly into `loss_of`.
        """
        d_query = None
        if self.d_mids is not None:
            if kwargs:
                raise ValueError(f"Expected no privacy arguments but got {kwargs}")
            if not self.d_mids:
                raise ValueError("Privacy allowance has been exhausted")
            d_query = self.d_mids[0]
        elif kwargs:
            measure, d_query = loss_of(**kwargs)
            if measure != self.output_measure:
                raise ValueError(
                    f"Expected output measure {self.output_measure} but got {measure}"
                )

        return Query(
            chain=(self.accountant.input_domain, self.accountant.input_metric),
            output_measure=self.accountant.output_measure,
            d_in=self.d_in,
            d_out=d_query,
            analysis=self,
        )


Chain = Union[Tuple[Domain, Metric], Transformation, Measurement, "PartialChain"]


class Query(object):
    """A helper API to build a measurement."""

    _chain: Chain
    """The current chain of transformations and measurements."""
    _output_measure: Measure
    """The output measure of the query."""
    _analysis: Optional["Analysis"]
    """The analysis that the query is part of. `query.release()` submits `_chain` to `_analysis`."""
    _wrap_release: Optional[Callable[[Any], Any]]
    """For internal use. A function that wraps the release of the query. 
    Used to wrap the response of compositor/odometer queries in another `Analysis`."""

    def __init__(
        self,
        chain: Chain,
        output_measure: Measure = None,
        d_in=None,
        d_out=None,
        analysis: "Analysis" = None,
        _wrap_release=None,
    ) -> None:
        """Initializes the query with the given chain and output measure.

        It is more convenient to use the `analysis.query()` constructor than this one.
        However, this can be used stand-alone to help build a transformation/measurement that is not part of an analysis.

        :param chain: an initial metric space (tuple of domain and metric) or transformation
        :param output_measure: how privacy will be measured on the output of the query
        :param d_in: an upper bound on the distance between adjacent datasets
        :param d_out: an upper bound on the overall privacy loss
        :param analysis: if specified, then when the query is released, the chain will be submitted to this analysis
        :param _wrap_release: for internal use only
        """
        self._chain = chain
        self._output_measure = output_measure
        self._d_in = d_in
        self._d_out = d_out
        self._analysis = analysis
        self._wrap_release = _wrap_release

    def __getattr__(self, name: str) -> Callable[[Any], "Query"]:
        """Creates a new query by applying a transformation or measurement to the current chain."""
        if name not in constructors:
            raise AttributeError(f"Unrecognized constructor: '{name}'")

        def make(*args, **kwargs):
            """Wraps the `make_{name}` constructor to allow one optional parameter and chains it to the current query.

            This function will be called when the user calls `query.{name}(...)`.
            """
            constructor, is_partial = constructors[name]

            # determine how many parameters are missing
            param_diff = len(args)
            for param in signature(constructor).parameters.values():
                if param.name in kwargs:
                    continue
                if param.default is not param.empty:
                    break
                param_diff -= 1

            if param_diff == -1 and not isinstance(self._chain, PartialChain):
                constructor = PartialChain.wrap(constructor)
            elif param_diff < 0:
                raise ValueError(f"{name} is missing {-param_diff} parameter(s).")
            elif param_diff > 0:
                raise ValueError(f"{name} has {param_diff} parameter(s) too many.")

            new_chain = constructor(*args, **kwargs)
            if is_partial or not isinstance(self._chain, tuple):
                new_chain = self._chain >> new_chain

            return self.new_with(chain=new_chain)

        return make

    def new_with(self, *, chain: Chain, wrap_release=None) -> "Query":
        """Convenience constructor that creates a new query with a different chain."""
        return Query(
            chain=chain,
            output_measure=self._output_measure,
            d_in=self._d_in,
            d_out=self._d_out,
            analysis=self._analysis,
            _wrap_release=wrap_release or self._wrap_release,
        )

    def __dir__(self):
        """Returns the list of available constructors. Used by Python's error suggestion mechanism."""
        return super().__dir__() + list(constructors.keys())

    def resolve(self, allow_transformations=False):
        """Resolve the query into a measurement."

        :param allow_transformations: If true, allow the response to be a transformation instead of a measurement.
        """
        # resolve a partial chain into a measurement, by fixing the input and output distances
        if isinstance(self._chain, PartialChain):
            chain = self._chain.fix(self._d_in, self._d_out)
            if chain.output_measure != self._output_measure:
                raise ValueError("Output measure does not match")
        else:
            chain = self._chain

        if not allow_transformations and isinstance(chain, Transformation):
            raise ValueError("Query is not yet a measurement")

        return chain

    def release(self) -> Any:
        """Release the query. The query must be part of an analysis."""
        # TODO: consider adding an optional `data` parameter for when _analysis is None
        answer = self._analysis(self.resolve())
        if self._wrap_release:
            answer = self._wrap_release(answer)
        return answer

    def param(self):
        """Returns the discovered parameter, if there is one"""
        return getattr(self.resolve(), "param", None)

    def zCDP_to_approxDP(
        self, map_query: Callable[["Query"], "Query"], delta=None
    ) -> "Query":
        """Converts a zCDP query to an approximate DP query.

        :param map_query: A function for constructing a zCDP query.
        :param delta: The target delta for the approximate DP query. Only use if no delta is specified in the query.
        """

        if delta is not None and self._d_out is not None:
            raise ValueError("`delta` has already been specified in query")
        if delta is None and self._d_out is None:
            raise ValueError("`delta` has not yet been specified in the query")
        delta = delta or self._d_out[1]

        new_measure = zero_concentrated_divergence(T=self._output_measure.type.args[0])

        def caster(measurement):
            return make_fix_delta(make_zCDP_to_approxDP(measurement), delta=delta)

        # convert from (eps, del) to rho
        d_out_rho = None
        if self._d_out:
            space = atom_domain(T=int), absolute_distance(T=float)
            scale = binary_search_param(
                lambda scale: caster(make_gaussian(*space, scale)),
                d_in=1.0,  # the choice of constant doesn't matter, so long as it is the same as below
                d_out=self._d_out,
                T=float,
            )
            d_out_rho = make_gaussian(*space, scale).map(1.0)

        return self._cast_measure(
            new_measure, d_out=d_out_rho, caster=caster, map_query=map_query
        )

    def pureDP_to_fixed_approxDP(
        self, map_query: Callable[["Query"], "Query"]
    ) -> "Query":
        """Converts a pure DP query to an approximate DP query.

        :param map_query: A function for constructing a pure DP query."""
        new_measure = fixed_smoothed_max_divergence(T=self._output_measure.type.args[0])

        def caster(measurement):
            return make_pureDP_to_fixed_approxDP(measurement)

        d_out_epsdel = (self._d_out, 0.0) if self._d_out else None
        return self._cast_measure(
            new_measure, d_out=d_out_epsdel, caster=caster, map_query=map_query
        )

    def pureDP_to_zCDP(self, map_query: Callable[["Query"], "Query"]) -> "Query":
        """Converts a pure DP query to a zCDP query.

        :param map_query: A function for constructing a pure DP query."""
        new_measure = zero_concentrated_divergence(T=self._output_measure.type.args[0])

        def caster(measurement):
            return make_pureDP_to_zCDP(measurement)

        # convert from rho to epsilon
        d_out_epsilon = None
        if self._d_out:
            scale = binary_search_param(
                lambda eps: caster(make_base_laplace(eps)),
                d_in=1.0,
                d_out=self._d_out,
                T=float,
            )
            d_out_epsilon = make_base_laplace(scale).map(1.0)

        return self._cast_measure(
            new_measure, d_out=d_out_epsilon, caster=caster, map_query=map_query
        )

    def _cast_measure(self, measure, d_out, caster, map_query):
        """Helper function for casting a measure."""
        inner_seed_query = Query(
            chain=self._chain,
            output_measure=measure,
            d_in=self._d_in,
            d_out=d_out,
        )
        inner_query = map_query(inner_seed_query)
        inner_chain = inner_query._chain

        # wrap the inner chain in the caster (leaving it partial if it was partial)
        if isinstance(inner_chain, PartialChain):
            casted_chain = PartialChain(lambda x: caster(inner_chain(x)))
        else:
            casted_chain = caster(inner_chain)

        return self.new_with(chain=casted_chain, wrap_release=inner_query._wrap_release)

    def sequential_composition(self, 
        split_evenly_over: Optional[int] = None,
        split_by_weights: Optional[List[float]] = None,
        d_out=None) -> "Analysis":
        """Constructs a new analysis containing a sequential compositor with the given weights.

        :param weights: A list of weights corresponding to the privacy budget allocated to a sequence of queries.
        """

        if d_out is not None and self._d_out is not None:
            raise ValueError("`d_out` has already been specified in query")
        if d_out is None and self._d_out is None:
            raise ValueError("`d_out` has not yet been specified in the query")
        d_out = d_out or self._d_out

        def compositor(chain: Union[Tuple[Domain, Metric], Transformation], d_in):
            if isinstance(chain, tuple):
                input_domain, input_metric = chain
            elif isinstance(chain, Transformation):
                input_domain, input_metric = chain.output_domain, chain.output_metric
                d_in = chain.map(d_in)

            privacy_unit = input_metric, d_in
            privacy_loss = self._output_measure, d_out

            accountant, d_mids = _sequential_composition_by_weights(
                input_domain, privacy_unit, privacy_loss, split_evenly_over, split_by_weights
            )
            if isinstance(chain, Transformation):
                accountant = chain >> accountant

            def wrap_release(queryable):
                return Analysis(
                    accountant=accountant,
                    queryable=queryable,
                    d_in=d_in,
                    d_mids=d_mids,
                )

            return self.new_with(chain=accountant, wrap_release=wrap_release)

        return self._compose_analysis(compositor)

    def _compose_analysis(self, compositor):
        """Helper function for composing an analysis."""
        if isinstance(self._chain, PartialChain):
            return PartialChain(lambda x: compositor(self._chain(x), self._d_in))
        else:
            return compositor(self._chain, self._d_in)


class PartialChain(object):
    """A partial chain is a transformation or measurement that is missing one numeric parameter.

    The parameter can be solved for by calling the fix method,
    which returns the closest transformation or measurement that satisfies the given stability or privacy constraint.
    """

    partial: Callable[[float], Union[Transformation, Measurement]]
    """The partial transformation or measurement."""

    def __init__(self, f, *args, **kwargs):
        self.partial = partial(f, *args, **kwargs)

    def __call__(self, v):
        """Returns the transformation or measurement with the given parameter."""
        return self.partial(v)

    def fix(self, d_in, d_out, T=None):
        """Returns the closest transformation or measurement that satisfies the given stability or privacy constraint.

        The discovered parameter is assigned to the param attribute of the returned transformation or measurement.
        """
        param = binary_search(lambda x: self.partial(x).check(d_in, d_out), T=T)
        chain = self.partial(param)
        chain.param = param
        return chain

    def __rshift__(self, other):
        # partials may be chained with other transformations or measurements to form a new partial
        if isinstance(other, (Transformation, Measurement)):
            return PartialChain(lambda x: self.partial(x) >> other)

        raise ValueError("At most one parameter may be missing at a time")

    @classmethod
    def wrap(cls, f):
        """Wraps a constructor for a transformation or measurement to return a partial chain instead."""

        def inner(*args, **kwargs):
            return cls(f, *args, **kwargs)

        return inner


def _sequential_composition_by_weights(
    domain: Domain,
    privacy_unit: Tuple[Metric, float],
    privacy_loss: Tuple[Measure, float],
    split_evenly_over: Optional[int] = None,
    split_by_weights: Optional[List[float]] = None,
) -> Tuple[Measurement, List[Any]]:
    """constructs a sequential composition measurement
    where the d_mids are proportional to the weights

    :param domain: the domain of the data
    :param privacy_unit: a tuple of the input metric and the data distance (d_in)
    :param privacy_loss: a tuple of the output measure and the privacy loss (d_out)
    :param weights: either a list of weights for each intermediate privacy loss, or the number of ways to evenly distribute the privacy loss
    """
    input_metric, d_in = privacy_unit
    output_measure, d_out = privacy_loss

    if split_evenly_over is not None and split_by_weights is not None:
        raise ValueError("Cannot specify both `split_evenly_over` and `split_by_weights`")

    if split_evenly_over is not None:
        weights = [d_out] * split_evenly_over
    elif split_by_weights is not None:
        weights = split_by_weights
    else:
        raise ValueError("Must specify either `split_evenly_over` or `split_by_weights`")

    def mul(dist, scale):
        if isinstance(dist, tuple):
            return dist[0] * scale, dist[1] * scale
        else:
            return dist * scale

    def scale_weights(scale, weights):
        return [mul(w, scale) for w in weights]

    def scale_sc(scale):
        return make_sequential_composition(
            input_domain=domain,
            input_metric=input_metric,
            output_measure=output_measure,
            d_in=d_in,
            d_mids=scale_weights(scale, weights),
        )

    scale = binary_search_param(scale_sc, d_in=d_in, d_out=d_out, T=float)

    # return the accountant and d_mids
    return scale_sc(scale), scale_weights(scale, weights)
