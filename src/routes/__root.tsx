import { createRootRoute, Outlet } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { ThemeProvider } from "#/components/theme-provider";
import { useLocale } from "#/lib/locale";
import { DrawerStackProvider } from "#/routes/(index)/-components/DrawerStack";
import "../styles.css";
import { TooltipProvider } from "#/components/ui/tooltip";

export const Route = createRootRoute({
	component: RootComponent,
});

function RootComponent() {
	// Re-render the whole component tree on locale change so m.*() message texts update immediately
	useLocale();
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
