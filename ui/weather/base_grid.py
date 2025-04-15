from gi.repository import Gtk


class BaseGrid(Gtk.Grid):
    def __init__(self, cols_count, rows_count, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.cols_count = cols_count
        self.rows_count = rows_count

        for col in range(0, self.cols_count):
            self.insert_column(col)
        for row in range(0, self.rows_count):
            self.insert_row(row)

    def update(self, data):
        for row in range(0, self.rows_count):
            self.update_row(data[row], row)
