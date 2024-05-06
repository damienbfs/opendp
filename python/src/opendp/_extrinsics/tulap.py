from opendp._extrinsics._utilities import to_then
from opendp.measurements import make_tulap
from opendp.mod import Measurement
import math
import numpy as np


def make_binomial_tulap(input_domain, input_metric, epsilon, delta, size) -> Measurement:
    return make_tulap(input_domain, input_metric, epsilon, delta) >> (
        lambda data: Tulap(data=data, epsilon=epsilon, delta=delta, size=size)
    )

then_binomial_tulap = to_then(make_binomial_tulap)


class Tulap(object):
    def __init__(self, data, epsilon, delta, size) -> None:
        self.data = data
        self.epsilon = epsilon
        self.delta = delta
        self.size = size

    def ump_test(self, theta, alpha, tail):
        return ump_test(
            self.data, theta, self.size, alpha, self.epsilon, self.delta, tail
        )

    def CI(self, alpha, tail=None):
        if tail is None:
            return CI_twoside(self.data, alpha, self.size, self.epsilon, self.delta)

        if tail in {"lower", "upper"}:
            return CI_oneside(
                self.data, alpha, self.size, self.epsilon, self.delta, tail
            )

        raise ValueError("tail must be None, lower or upper")

    def p_value(self, theta, tail=None):
        if tail is None:
            return twoside_pvalue(self.data, theta, self.size, self.epsilon, self.delta)

        if tail in {"left", "right"}:
            return oneside_pvalue(
                self.data, theta, self.size, self.epsilon, self.delta, tail
            )

        raise ValueError("tail must be None, left or right")
    
    def __repr__(self) -> str:
        return f"Tulap({self.data})"



def _ptulap(t, m=0, epsilon=0, delta=0):
    t = np.atleast_1d(t)
    b = math.exp(-epsilon)
    q = (2 * delta * b) / (1 - b + 2 * delta * b)
    lcut = q / 2
    rcut = q / 2
    t = t - m  # normalize
    r = np.rint(t)
    g = -math.log(b)
    l = math.log(1 + b)
    k = 1 - b

    negs = np.exp((r * g) - l + np.log(b + ((t - r + (1 / 2)) * k)))
    poss = 1 - np.exp((r * -g) - l + np.log(b + ((r - t + (1 / 2)) * k)))

    # check for infinities
    negs[np.isinf(negs)] = 0
    poss[np.isinf(poss)] = 0
    # truncate w.r.t. the indicator on t's positivity
    is_leq0 = np.less_equal(t, 0).astype(int)
    trunc = (is_leq0 * negs) + ((1 - is_leq0) * poss)

    # handle the cut adjustment and scaling
    q = lcut + rcut
    is_mid = np.logical_and(
        np.less_equal(lcut, trunc), np.less_equal(trunc, (1 - rcut))
    ).astype(int)
    is_rhs = np.less((1 - rcut), trunc).astype(int)
    return ((trunc - lcut) / (1 - q)) * is_mid + is_rhs


def ump_test(data, theta, size, alpha, epsilon, delta, tail):
    from scipy.stats import binom
    import scipy

    values = list(range(0, size + 1))
    B = binom.pmf(k=values, n=size, p=theta)

    def obj(s):
        values_array = np.array(values)
        phi = _ptulap(t=values_array - s, epsilon=epsilon, delta=delta)
        return np.dot(B, phi) - alpha

    lower = -1
    upper = 1

    while obj(lower) < 0:
        lower *= 2
    while obj(upper) > 0:
        upper *= 2
    root = scipy.optimize.brentq(obj, lower, upper)
    s = root
    values_array = np.array(values)
    phi = _ptulap(t=values_array - s, epsilon=epsilon, delta=delta)

    if np.any(data) and tail == "left":
        return phi

    if np.any(data) and tail == "right":
        return 1 - phi


def oneside_pvalue(Z, theta, size, epsilon, delta, tail):
    """Right tailed p-value

    :param Z: tulap random variables
    :param theta: true probability of binomial distribution
    :param size: number of trials
    """
    from scipy.stats import binom

    values = np.array(range(size))
    B = binom.pmf(k=values, n=size, p=theta)
    if tail == "right":
        F = _ptulap(t=values - Z, m=0, epsilon=epsilon, delta=delta)
    elif tail == "left":
        F = 1 - _ptulap(t=values - Z, m=0, epsilon=epsilon, delta=delta)
    return np.dot(F.T, B)


def twoside_pvalue(Z, theta, size, epsilon, delta):
    Z = np.array(Z)

    T = abs(Z - size * theta)
    pval_right = oneside_pvalue(theta, size, epsilon, delta, "right")(T + size * theta)
    pval_left = oneside_pvalue(theta, size, epsilon, delta, "right")(size * theta - T)

    return np.subtract(pval_right, pval_left) + 1


def CI_oneside(Z, alpha, size, epsilon, delta, tail):
    from scipy.optimize import OptimizeResult, minimize_scalar  # type: ignore[import]

    def custmin(
        fun,
        bracket,
        args=(),
        maxfev=None,
        stepsize=1e-3,
        maxiter=500,
        callback=None,
        **options
    ):
        print("binary search, stepsize = ", 1e-3)
        lower = bracket[0]
        upper = bracket[1]

        funcalls = 1
        niter = 0

        mid = (lower + upper) / 2.0
        bestx = mid
        besty = fun(mid, *args)
        min_diff = 1e-6

        while lower <= upper:
            mid = (lower + upper) / 2.00
            # print("low: ", lower, "up: ", upper)
            # print("mid: ", mid)
            # print("diff: ", fun(mid, *args))
            funcalls += 1
            niter += 1
            if fun(mid, *args) == 0:
                # print("diff = 0")
                besty = fun(mid, *args)
                bestx = mid
                return OptimizeResult(
                    fun=besty, x=bestx, nit=niter, nfev=funcalls, success=(niter > 1)
                )
            elif abs(fun(mid, *args)) <= min_diff:
                # print("diff <= min_diff")
                besty = fun(mid, *args)
                bestx = mid
                return OptimizeResult(
                    fun=besty, x=bestx, nit=niter, nfev=funcalls, success=(niter > 1)
                )
            elif fun(mid, *args) > 0:  # mid > alpha
                # print("diff > 0")
                upper = mid - stepsize
            elif fun(mid, *args) < 0:  # mid < alpha
                # print("diff < 0")
                lower = mid + stepsize

        bestx = mid
        besty = fun(mid, *args)
        # print("while loop break")
        # print("low and up: ", lower, upper)
        return OptimizeResult(
            fun=besty, x=bestx, nit=niter, nfev=funcalls, success=(niter > 1)
        )

    Z = np.array(Z)
    if tail == "lower":
        CIobj = (
            lambda x: (
                oneside_pvalue(
                    x, size=size, epsilon=epsilon, delta=delta, tail="right"
                )(Z)[0]
            )
            - alpha
        )

    elif tail == "upper":
        CIobj = lambda x: (
            (
                oneside_pvalue(
                    x, size=size, epsilon=epsilon, delta=delta, tail="right"
                )(Z)[0]
            )
            - (1 - alpha)
        )
    return minimize_scalar(fun=CIobj, method=custmin, bracket=(0, 1)).x


def CI_twoside(Z, alpha, size, epsilon, delta):
    from scipy.optimize import OptimizeResult, minimize_scalar

    def custmin(
        fun,
        bracket,
        args=(),
        maxfev=None,
        stepsize=1e-3,
        maxiter=500,
        callback=None,
        **options
    ):
        print("binary search, stepsize = ", 1e-3)
        lower = bracket[0]
        upper = bracket[1]

        funcalls = 1
        niter = 0

        mid = (lower + upper) / 2.0
        bestx = mid
        besty = fun(mid, *args)
        min_diff = 1e-6

        while lower <= upper:
            mid = (lower + upper) / 2.00
            print("low: ", lower, "up: ", upper)
            print("mid: ", mid)
            print("diff: ", fun(mid, *args))
            funcalls += 1
            niter += 1
            if fun(mid, *args) == 0:
                print("diff = 0")
                besty = fun(mid, *args)
                bestx = mid
                return OptimizeResult(
                    fun=besty, x=bestx, nit=niter, nfev=funcalls, success=(niter > 1)
                )
            elif abs(fun(mid, *args)) <= min_diff:
                print("diff <= min_diff")
                besty = fun(mid, *args)
                bestx = mid
                return OptimizeResult(
                    fun=besty, x=bestx, nit=niter, nfev=funcalls, success=(niter > 1)
                )
            elif fun(mid, *args) > 0:  # mid > alpha
                print("diff > 0")
                upper = mid - stepsize
            elif fun(mid, *args) < 0:  # mid < alpha
                print("diff < 0")
                lower = mid + stepsize

        bestx = mid
        besty = fun(mid, *args)
        print("while loop break")
        print("low and up: ", lower, upper)
        return OptimizeResult(
            fun=besty, x=bestx, nit=niter, nfev=funcalls, success=(niter > 1)
        )

    print("step 1: inside function z")
    Z = np.array(Z) if not isinstance(Z, np.ndarray) else Z
    mle = Z / size
    mle = max(min(mle, 1), 0)
    print("step 2: calc z")
    # twoside_pvalue_func = make_twoside_pvalue(theta=mle, size=size, epsilon=epsilon, delta=delta)
    print("step 3: calc twoside_pvalue_func")
    # CIobj2 = lambda x: (twoside_pvalue_func(np.array([Z]))[0] - alpha)
    CIobj2 = lambda x: (
        twoside_pvalue(theta=mle, size=size, epsilon=epsilon, delta=delta)(
            np.array([x])
        )[0]
        - alpha
    )

    if mle > 0:
        L = minimize_scalar(fun=CIobj2, method=custmin, bracket=(0, mle))
        L = L.x
    else:
        L = 0

    if mle < 1:
        U = minimize_scalar(fun=CIobj2, method=custmin, bracket=(mle, 1))
        U = U.x
    else:
        U = 1

    return [L, U]
