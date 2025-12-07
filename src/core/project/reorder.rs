use aoer_plotty_rs::plotter::pen::PenDetail;

impl super::Project {
    /// Reorders the geometry by the Pen Tool ID.
    pub fn reorder_geometry_by_tool_id(&mut self) {
        let mut working_geo = self.plot_geometry.clone();
        working_geo.sort_by(|geo1, geo2| {
            let pen1 = self
                .pen_by_uuid(geo1.pen_uuid)
                .unwrap_or(PenDetail::default());
            let pen2 = self
                .pen_by_uuid(geo2.pen_uuid)
                .unwrap_or(PenDetail::default());
            pen1.tool_id.cmp(&pen2.tool_id)
        });
        self.plot_geometry = working_geo;
    }
}
