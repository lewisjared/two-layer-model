# ---
# jupyter:
#   jupytext:
#     text_representation:
#       extension: .py
#       format_name: percent
#       format_version: '1.3'
#       jupytext_version: 1.16.4
#   kernelspec:
#     display_name: Python 3 (ipykernel)
#     language: python
#     name: python3
# ---

# %% [markdown]
# # State serialisation
#

# %%
from helpers.models import example_model_builder

from two_layer_model._lib.core import Model

# %%

model = example_model_builder(1750, 2100).build()
model.step()
model.step()

# %% [markdown]
# We can log the state of a model using JSON.

serialised_model = model.to_json()
serialised_model

# %%
new_model = Model.from_json(serialised_model)
new_model
