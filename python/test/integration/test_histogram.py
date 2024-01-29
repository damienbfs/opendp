import pytest
import opendp.prelude as dp

dp.enable_features("contrib", "floating-point")


def test_count_by_categories():
    """Compute histogram with known category set"""
    preprocess = (
        dp.t.make_split_dataframe(",", ["A", "B"])
        >> dp.t.make_select_column("A", TOA=str)
        >> dp.t.then_count_by_categories(
            categories=["a", "b", "c"], MO=dp.L1Distance[int]
        )
    )

    noisy_histogram_from_dataframe = dp.binary_search_chain(
        lambda s: preprocess >> dp.m.then_base_discrete_laplace(s), d_in=1, d_out=1.0
    )

    assert noisy_histogram_from_dataframe.check(1, 1.0)

    data = "\n".join(["a"] * 25 + ["b"] * 25 + ["what?"] * 10)

    print(noisy_histogram_from_dataframe(data))


def test_count_by_categories_float():
    """Compute histogram with known category set"""
    data = "\n".join(["a"] * 5 + ["b"] * 20 + ["c"] * 10 + ["z"] * 5)
    cats = ["a", "b", "c"]
    load = dp.t.make_split_dataframe(",", ["A", "B"]) >> dp.t.make_select_column(
        "A", TOA=str
    )

    t_hist = load >> dp.t.then_count_by_categories(
        cats, MO=dp.L1Distance[float], TOA=float
    )
    assert t_hist(data) == [5.0, 20.0, 10.0, 5.0]
    # ensure that chaining works as expected
    (t_hist >> dp.m.then_laplace(1.0))(data)

    t_hist = load >> dp.t.then_count_by_categories(
        cats, MO=dp.L2Distance[float], TOA=float
    )
    assert t_hist(data) == [5.0, 20.0, 10.0, 5.0]
    (t_hist >> dp.m.then_gaussian(1.0))(data)


def test_count_by_threshold():
    """Compute histogram with unknown category set"""
    pre = (
        dp.t.make_split_dataframe(",", ["A", "B"])
        >> dp.t.make_select_column("A", TOA=str)
        >> dp.t.then_count_by(MO=dp.L1Distance[float], TV=float)
    )
    budget = (1.0, 1e-8)

    scale = dp.binary_search_param(
        lambda s: pre >> dp.m.then_base_laplace_threshold(scale=s, threshold=1e8),
        d_in=1,
        d_out=budget,
    )
    threshold = dp.binary_search_param(
        lambda t: pre >> dp.m.then_base_laplace_threshold(scale=scale, threshold=t),
        d_in=1,
        d_out=budget,
    )

    laplace_histogram_from_dataframe = pre >> dp.m.then_base_laplace_threshold(
        scale=scale, threshold=threshold
    )

    assert laplace_histogram_from_dataframe.check(1, budget)

    data = "\n".join(["a"] * 500 + ["b"] * 200 + ["other"] * 100)

    assert pre(data) == {"a": 500, "b": 200, "other": 100}
    print(laplace_histogram_from_dataframe(data))
    print(scale, threshold)

    with pytest.raises(dp.OpenDPException):
        dp.m.make_base_laplace_threshold(
            dp.atom_domain(T=int), dp.l1_distance(T=float), scale=1.0, threshold=1e8
        )

    extreme_vals = pre >> dp.m.then_base_laplace_threshold(
        scale=0., threshold=threshold
    )
    print(extreme_vals.map(1))

test_count_by_threshold()