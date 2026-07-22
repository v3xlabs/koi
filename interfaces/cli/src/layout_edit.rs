//! Pure account-layout editing, mirroring the web frontend's drag logic:
//! moves reassign contiguous 0-based `display_order` inside every affected
//! bucket, and saving always sends the complete layout.

use koi::models::account::{
    Account,
    group::{AccountGroup, GroupIdentity},
    layout::{AccountLayoutAccountEntry, AccountLayoutGroupEntry, AccountLayoutUpdate},
};

use super::app::normalized_group_id;

pub fn bucket_ids(accounts: &[Account], group_id: Option<u64>) -> Vec<u64> {
    let mut bucket: Vec<&Account> = accounts
        .iter()
        .filter(|account| normalized_group_id(account) == group_id)
        .collect();
    bucket.sort_by_key(|account| (account.display_order, account.account_identity.0));
    bucket
        .into_iter()
        .map(|account| account.account_identity.0)
        .collect()
}

fn set_bucket_orders(accounts: &mut [Account], ordered_ids: &[u64]) {
    for (index, id) in ordered_ids.iter().enumerate() {
        if let Some(account) = accounts
            .iter_mut()
            .find(|account| account.account_identity.0 == *id)
        {
            account.display_order = index as u32;
        }
    }
}

pub fn move_account_within(accounts: &mut [Account], account_id: u64, delta: i32) -> bool {
    let group_id = accounts
        .iter()
        .find(|account| account.account_identity.0 == account_id)
        .map(normalized_group_id);
    let Some(group_id) = group_id else {
        return false;
    };

    let mut ordered = bucket_ids(accounts, group_id);
    let Some(position) = ordered.iter().position(|id| *id == account_id) else {
        return false;
    };
    let target = (position as i32 + delta).clamp(0, ordered.len() as i32 - 1) as usize;
    if target == position {
        return false;
    }

    ordered.remove(position);
    ordered.insert(target, account_id);
    set_bucket_orders(accounts, &ordered);
    true
}

pub fn move_account_to_group(
    accounts: &mut [Account],
    account_id: u64,
    target_group: Option<u64>,
    index: usize,
) -> bool {
    let source_group = accounts
        .iter()
        .find(|account| account.account_identity.0 == account_id)
        .map(normalized_group_id);
    let Some(source_group) = source_group else {
        return false;
    };
    if source_group == target_group {
        return false;
    }

    if let Some(account) = accounts
        .iter_mut()
        .find(|account| account.account_identity.0 == account_id)
    {
        account.group_id = target_group.map(GroupIdentity);
    }

    let mut target_ids = bucket_ids(accounts, target_group);
    target_ids.retain(|id| *id != account_id);
    target_ids.insert(index.min(target_ids.len()), account_id);
    set_bucket_orders(accounts, &target_ids);

    let source_ids = bucket_ids(accounts, source_group);
    set_bucket_orders(accounts, &source_ids);
    true
}

pub fn move_group(groups: &mut [AccountGroup], group_id: u64, delta: i32) -> bool {
    let mut ordered: Vec<u64> = {
        let mut sorted: Vec<&AccountGroup> = groups.iter().collect();
        sorted.sort_by_key(|group| (group.display_order, group.group_identity.0));
        sorted.iter().map(|group| group.group_identity.0).collect()
    };
    let Some(position) = ordered.iter().position(|id| *id == group_id) else {
        return false;
    };
    let target = (position as i32 + delta).clamp(0, ordered.len() as i32 - 1) as usize;
    if target == position {
        return false;
    }

    ordered.remove(position);
    ordered.insert(target, group_id);
    for (index, id) in ordered.iter().enumerate() {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.group_identity.0 == *id)
        {
            group.display_order = index as u32;
        }
    }
    true
}

pub fn build_layout_update(groups: &[AccountGroup], accounts: &[Account]) -> AccountLayoutUpdate {
    let mut sorted_groups: Vec<&AccountGroup> = groups.iter().collect();
    sorted_groups.sort_by_key(|group| (group.display_order, group.group_identity.0));

    let group_entries = sorted_groups
        .iter()
        .enumerate()
        .map(|(index, group)| AccountLayoutGroupEntry {
            group_identity: group.group_identity,
            name: group.name.clone(),
            display_order: index as u32,
        })
        .collect();

    let mut account_entries = Vec::with_capacity(accounts.len());
    let buckets = sorted_groups
        .iter()
        .map(|group| Some(group.group_identity.0))
        .chain(std::iter::once(None));
    for bucket in buckets {
        for (index, id) in bucket_ids(accounts, bucket).into_iter().enumerate() {
            account_entries.push(AccountLayoutAccountEntry {
                account_identity: koi::models::account::identity::AccountIdentity(id),
                group_id: bucket.map(GroupIdentity),
                display_order: index as u32,
            });
        }
    }

    AccountLayoutUpdate {
        groups: group_entries,
        accounts: account_entries,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn account(id: u64, group: Option<u64>, order: u32) -> Account {
        serde_json::from_value(json!({
            "account_identity": id,
            "name": format!("acc-{id}"),
            "networks": [],
            "metadata": {"type": "view", "evm_address": "0x0000000000000000000000000000000000000000"},
            "group_id": group,
            "display_order": order,
        }))
        .unwrap()
    }

    fn group(id: u64, order: u32) -> AccountGroup {
        AccountGroup {
            group_identity: GroupIdentity(id),
            name: format!("group-{id}"),
            display_order: order,
        }
    }

    #[test]
    fn moves_within_a_bucket_and_renumbers() {
        let mut accounts = vec![
            account(1, Some(7), 0),
            account(2, Some(7), 1),
            account(3, Some(7), 2),
            account(9, None, 0),
        ];

        assert!(move_account_within(&mut accounts, 3, -1));
        assert_eq!(bucket_ids(&accounts, Some(7)), vec![1, 3, 2]);
        assert!(!move_account_within(&mut accounts, 1, -1));
        assert_eq!(bucket_ids(&accounts, None), vec![9]);

        let orders: Vec<u32> = bucket_ids(&accounts, Some(7))
            .iter()
            .map(|id| {
                accounts
                    .iter()
                    .find(|account| account.account_identity.0 == *id)
                    .unwrap()
                    .display_order
            })
            .collect();
        assert_eq!(orders, vec![0, 1, 2]);
    }

    #[test]
    fn moves_across_buckets_renumbering_both() {
        let mut accounts = vec![
            account(1, Some(7), 0),
            account(2, Some(7), 1),
            account(3, None, 0),
        ];

        assert!(move_account_to_group(&mut accounts, 1, None, 0));
        assert_eq!(bucket_ids(&accounts, Some(7)), vec![2]);
        assert_eq!(bucket_ids(&accounts, None), vec![1, 3]);
        assert_eq!(
            accounts
                .iter()
                .find(|account| account.account_identity.0 == 2)
                .unwrap()
                .display_order,
            0
        );
    }

    #[test]
    fn reorders_groups() {
        let mut groups = vec![group(1, 0), group(2, 1), group(3, 2)];

        assert!(move_group(&mut groups, 3, -2));
        let ordered: Vec<u64> = {
            let mut sorted: Vec<&AccountGroup> = groups.iter().collect();
            sorted.sort_by_key(|group| (group.display_order, group.group_identity.0));
            sorted.iter().map(|group| group.group_identity.0).collect()
        };
        assert_eq!(ordered, vec![3, 1, 2]);
        assert!(!move_group(&mut groups, 3, -1));
    }

    #[test]
    fn layout_update_covers_every_group_and_account() {
        let accounts = vec![
            account(1, Some(7), 4),
            account(2, None, 9),
            account(3, Some(7), 1),
        ];
        let groups = vec![group(7, 3)];

        let update = build_layout_update(&groups, &accounts);
        assert_eq!(update.groups.len(), 1);
        assert_eq!(update.groups[0].display_order, 0);
        assert_eq!(update.accounts.len(), 3);

        let entry = |id: u64| {
            update
                .accounts
                .iter()
                .find(|entry| entry.account_identity.0 == id)
                .unwrap()
        };
        assert_eq!(entry(3).display_order, 0);
        assert_eq!(entry(1).display_order, 1);
        assert_eq!(entry(2).group_id, None);
        assert_eq!(entry(2).display_order, 0);
    }

    #[test]
    fn treats_group_zero_as_ungrouped() {
        let accounts = vec![account(1, Some(0), 0), account(2, None, 1)];
        assert_eq!(bucket_ids(&accounts, None), vec![1, 2]);
    }
}
