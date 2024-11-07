use crate::frontends::components::heading::Heading;
use crate::frontends::components::sub_heading::Subheading;
use crate::frontends::components::tables::table::Table;
use crate::frontends::components::tables::table_cell::TableCell;
use crate::frontends::components::tables::table_head::TableHead;
use crate::frontends::components::tables::table_header::TableHeader;
use block_mesh_common::interfaces::server_api::DailyLeaderboard;
use leptos::*;

#[component]
pub fn DailyLeaderboardDashboard() -> impl IntoView {
    let data = expect_context::<DailyLeaderboard>();
    let day = data.day.to_string();
    let users = Signal::derive(move || data.leaderboard_users.clone());

    view! {
        <div class="flex items-start justify-start gap-4">
            <Heading>Top 5 <span class="pr-2 pl-2">-</span> Daily Leaderboard</Heading>
        </div>
        <Subheading class="mt-14">
            Daily Users Ranking <span class="pr-2 pl-2">|</span> {day}
            <span class="pr-2 pl-2">|</span> not including perks or referal bonus
        </Subheading>
        <Table class="mt-4 [--gutter:theme(spacing.6)] lg:[--gutter:theme(spacing.10)]">
            <TableHead>
                <tr>
                    <TableHeader>Rank</TableHeader>
                    <TableHeader>Email</TableHeader>
                    // <TableHeader>IPs</TableHeader>
                    <TableHeader class="text-right">Points</TableHeader>
                </tr>
            </TableHead>

            <tbody>
                <Suspense>
                    {users
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(index, user)| {
                            view! {
                                <tr>
                                    <TableCell>{index + 1}</TableCell>
                                    <TableCell>{user.email.clone()}</TableCell>
                                    // <TableCell>{user.ips.unwrap_or_default()}</TableCell>
                                    <TableCell class="text-right">
                                        {format!("{:.1}", user.points.unwrap_or_default())}
                                    </TableCell>
                                </tr>
                            }
                        })
                        .collect_view()}
                </Suspense>
            </tbody>
        </Table>
    }
}
