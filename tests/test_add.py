import pytest

from two_layer_model._lib import add, solve_tlm


@pytest.mark.parametrize(
    "a, b, expected",
    [[1, 2, 3], [2, 3, 5], [1.3, 2.7, 4.0], [1, 2.7, 3.7]],
)
def test_add(a, b, expected):
    assert add(1, 2) == 3


@pytest.mark.parametrize("a", ["1", None])
def test_add_invalid_types(a):
    with pytest.raises(TypeError):
        add(a, 1)

    with pytest.raises(TypeError):
        add(1, a)


def test_solve():
    solve_tlm()
