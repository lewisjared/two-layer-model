"""
Rust Simple Climate Model (RSCM)

A framework for simple climate models built it Rust.
"""

import importlib.metadata

from ._lib import TwoLayerComponentBuilder  # noqa

__version__ = importlib.metadata.version("rscm")
