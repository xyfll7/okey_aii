// import { TanStackDevtools } from '@tanstack/react-devtools'
import { createRootRoute, Outlet } from "@tanstack/react-router";
// import { TanStackRouterDevtoolsPanel } from '@tanstack/react-router-devtools'

import "../styles.css";
import { ChatInit } from "#/components/chat/chatInit";
import { ChatProvider } from "#/components/chat/chatProvider";
import { DrawerStackProvider } from "#/components/drawer-stack";
import { ThemeProvider } from "#/components/theme-provider";

export const Route = createRootRoute({
	component: RootComponent,
});

function RootComponent() {
	return (
		<ThemeProvider defaultTheme="system" storageKey="vite-ui-theme">
			<DrawerStackProvider>
				<ChatProvider>
					<ChatInit>
						<Outlet />
					</ChatInit>
				</ChatProvider>
			</DrawerStackProvider>
			{/* <TanStackDevtools
        config={{
          position: 'bottom-right',
        }}
        plugins={[
          {
            name: 'TanStack Router',
            render: <TanStackRouterDevtoolsPanel />,
          },
        ]}
      /> */}
		</ThemeProvider>
	);
}
