from two_layer_model.core import TimeseriesCollection, VariableType


class TestTimeseriesCollection:
    def test_create(self, timeseries):
        collection = TimeseriesCollection()
        collection.add_timeseries("Test", timeseries, VariableType.Exogenous)
        collection.add_timeseries("Other", timeseries, VariableType.Endogenous)

        assert repr(collection) == '<TimeseriesCollection names=["Test", "Other"]>'
