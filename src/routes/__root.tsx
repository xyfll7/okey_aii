import { createRootRoute, Outlet } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { ThemeProvider } from "#/components/theme-provider";
import { DrawerStackProvider } from "#/routes/(index)/-components/DrawerStack";
import "../styles.css";
import { TooltipProvider } from "#/components/ui/tooltip";

export const Route = createRootRoute({
	component: RootComponent,
});

function RootComponent() {
	return (
		<ThemeProvider defaultTheme="system" storageKey="vite-ui-theme">
			<TooltipProvider>
				<DrawerStackProvider
					onClose={(ids) => {
						for (const session_id of ids) {
							invoke("close_session", { session_id }).catch(console.error);
						}
					}}
				>
					<Outlet />
				</DrawerStackProvider>
			</TooltipProvider>
		</ThemeProvider>
	);
}
