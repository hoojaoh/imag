//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2019 Matthias Beyer <mail@beyermatthias.de> and contributors
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; version
// 2.1 of the License.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//

use std::collections::HashMap;

use libimagstore::store::Store;
use libimagstore::storeid::StoreId;
use libimagutil::debug_result::DebugResult;

use failure::ResultExt;
use failure::Fallible as Result;
use failure::Error;
use failure::err_msg;

use crate::linker::*;

pub trait StoreLinkConsistentExt {
    fn check_link_consistency(&self) -> Result<()>;
}

impl StoreLinkConsistentExt for Store {
    fn check_link_consistency(&self) -> Result<()> {
        // Helper data structure to collect incoming and outgoing links for each StoreId
        #[derive(Debug, Default)]
        struct Linking {
            outgoing: Vec<StoreId>,
            incoming: Vec<StoreId>,
        }

        // Helper function to aggregate the Link network
        //
        // This function aggregates a HashMap which maps each StoreId object in the store onto
        // a Linking object, which contains a list of StoreIds which this entry links to and a
        // list of StoreIds which link to the current one.
        //
        // The lambda returns an error if something fails
        let aggregate_link_network = |store: &Store| -> Result<HashMap<StoreId, Linking>> {
            store
                .entries()?
                .into_get_iter()
                .fold(Ok(HashMap::new()), |map, element| {
                    map.and_then(|mut map| {
                        debug!("Checking element = {:?}", element);
                        let entry = element?.ok_or_else(|| err_msg("TODO: Not yet handled"))?;

                        debug!("Checking entry = {:?}", entry.get_location());

                        let internal_links = entry
                            .links()?
                            .into_getter(store); // get the FLEs from the Store

                        let mut linking = Linking::default();
                        for internal_link in internal_links {
                            debug!("internal link = {:?}", internal_link);

                            linking.outgoing.push(internal_link?.get_location().clone());
                            linking.incoming.push(entry.get_location().clone());
                        }

                        map.insert(entry.get_location().clone(), linking);
                        Ok(map)
                    })
                })
        };

        // Helper to check whethre all StoreIds in the network actually exists
        //
        // Because why not?
        let all_collected_storeids_exist = |network: &HashMap<StoreId, Linking>| -> Result<()> {
            for (id, _) in network.iter() {
                if is_match!(self.get(id.clone()), Ok(Some(_))) {
                    debug!("Exists in store: {:?}", id);

                    if !self.exists(id.clone())? {
                        warn!("Does exist in store but not on FS: {:?}", id);
                        return Err(err_msg("Link target does not exist"))
                    }
                } else {
                    warn!("Does not exist in store: {:?}", id);
                    return Err(err_msg("Link target does not exist"))
                }
            }

            Ok(())
        };

        // Helper function to create a SLCECD::OneDirectionalLink error object
        let mk_one_directional_link_err = |src: StoreId, target: StoreId| -> Error {
            Error::from(format_err!("Dead link: {} -> {}",
                                    src.local_display_string(),
                                    target.local_display_string()))
        };

        // Helper lambda to check whether the _incoming_ links of each entry actually also
        // appear in the _outgoing_ list of the linked entry
        let incoming_links_exists_as_outgoing_links =
            |src: &StoreId, linking: &Linking, network: &HashMap<StoreId, Linking>| -> Result<()> {
                for link in linking.incoming.iter() {
                    // Check whether the links which are _incoming_ on _src_ are outgoing
                    // in each of the links in the incoming list.
                    let incoming_consistent = network.get(link)
                        .map(|l| l.outgoing.contains(src))
                        .unwrap_or(false);

                    if !incoming_consistent {
                        return Err(mk_one_directional_link_err(src.clone(), link.clone()))
                    }
                }

                Ok(())
            };

        // Helper lambda to check whether the _outgoing links of each entry actually also
        // appear in the _incoming_ list of the linked entry
        let outgoing_links_exist_as_incoming_links =
            |src: &StoreId, linking: &Linking, network: &HashMap<StoreId, Linking>| -> Result<()> {
                for link in linking.outgoing.iter() {
                    // Check whether the links which are _outgoing_ on _src_ are incoming
                    // in each of the links in the outgoing list.
                    let outgoing_consistent = network.get(link)
                        .map(|l| l.incoming.contains(src))
                        .unwrap_or(false);

                    if !outgoing_consistent {
                        return Err(mk_one_directional_link_err(link.clone(), src.clone()))
                    }
                }

                Ok(())
            };

        aggregate_link_network(&self)
            .map_dbg_str("Aggregated")
            .map_dbg(|nw| {
                let mut s = String::new();
                for (k, v) in nw {
                    s.push_str(&format!("{}\n in: {:?}\n out: {:?}", k, v.incoming, v.outgoing));
                }
                s
            })
            .and_then(|nw| {
                all_collected_storeids_exist(&nw)
                    .map(|_| nw)
                    .context(err_msg("Link handling error"))
                    .map_err(Error::from)
            })
            .and_then(|nw| {
                for (id, linking) in nw.iter() {
                    incoming_links_exists_as_outgoing_links(id, linking, &nw)?;
                    outgoing_links_exist_as_incoming_links(id, linking, &nw)?;
                }
                Ok(())
            })
            .map(|_| ())
    }
}

