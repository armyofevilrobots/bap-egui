use std::collections::BTreeSet;

use aoer_plotty_rs::plotter::pen::PenDetail;

use crate::core::project::BAPGeometry;

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

    /// This will take the selected geometry and put it (in it's current order) at the selected
    /// location. If multiple geometries are selected, they will ALL move in their current order
    /// to a block at the new location, pushing all later geometries behind them.
    pub fn reorder_selected_to(
        &mut self,
        selection: &BTreeSet<u32>,
        destination: usize,
    ) -> BTreeSet<u32> {
        // Don't bother if nothing is selected
        let mut new_picked: BTreeSet<u32> = BTreeSet::new();
        if let Some(first) = selection.first()
            && let Some(last) = selection.last()
            && destination <= self.plot_geometry.len()
        // Has to be in range, or at the end.
        {
            let destination = if destination > *last as usize {
                destination + 1
            } else {
                destination
            };
            let mut selected: Vec<BAPGeometry> = (&selection)
                .into_iter()
                .map(|idx| self.plot_geometry[*idx as usize].clone())
                .collect();
            let mut new_geometry: Vec<BAPGeometry> = Vec::new();
            // Just iter and pick thems.
            let mut offset: usize = 0;
            for idx in 0..self.plot_geometry.len() {
                if idx == destination {
                    new_geometry.append(&mut selected);
                    // Also create a new pick list!
                    for i in (idx - offset)..(idx + selection.len() - offset) {
                        new_picked.insert(i as u32);
                    }
                }
                if (idx as u32) < *first || (idx as u32) > *last {
                    new_geometry.push(self.plot_geometry[idx].clone());
                } else {
                    // We're in the selected zone, only insert if its not selected
                    if !selection.contains(&(idx as u32)) {
                        new_geometry.push(self.plot_geometry[idx].clone());
                    } else {
                        offset += 1;
                    }
                }
            }
            //Special case. If it was being added at the end:
            if destination >= self.plot_geometry.len() {
                new_geometry.append(&mut selected);
                for i in (self.plot_geometry.len() - offset)..self.plot_geometry.len() {
                    new_picked.insert(i as u32);
                }
            }
            self.plot_geometry = new_geometry;
            new_picked
        } else {
            selection.clone()
        }
    }
}
