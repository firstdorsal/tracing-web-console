import { ThemeProvider } from "@/components/theme-provider";
import { createRoot } from "react-dom/client";
import App from "./App.tsx";
import "./index.css";

createRoot(document.getElementById("root")!).render(
    <ThemeProvider defaultTheme="dark" storageKey="tracing-ui-theme">
        <App />
    </ThemeProvider>
);
