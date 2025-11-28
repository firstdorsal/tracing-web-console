import { Badge } from "@/components/ui/badge";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue
} from "@/components/ui/select";
import type { LogLevel } from "@/types/logs";
import { useEffect, useState } from "react";
import { Settings } from "lucide-react";

interface TargetLevel {
    target: string;
    level: LogLevel;
}

interface LevelControlsProps {
    className?: string;
}

const LOG_LEVELS: LogLevel[] = ["trace", "debug", "info", "warn", "error"];

export function LevelControls({ className }: LevelControlsProps) {
    const [targets, setTargets] = useState<string[]>([]);
    const [targetLevels, setTargetLevels] = useState<Map<string, LogLevel>>(new Map());
    const [loading, setLoading] = useState(false);
    const [feedback, setFeedback] = useState<{
        type: "success" | "error";
        message: string;
    } | null>(null);

    // Fetch available targets
    useEffect(() => {
        const fetchTargets = async () => {
            try {
                const response = await fetch("api/targets");
                if (!response.ok) throw new Error("Failed to fetch targets");
                const data = await response.json();
                setTargets(data.targets || []);
            } catch (err) {
                console.error("Error fetching targets:", err);
                setFeedback({
                    type: "error",
                    message: "Failed to load targets"
                });
            }
        };

        fetchTargets();
        // Refresh targets every 10 seconds
        const interval = setInterval(fetchTargets, 10000);
        return () => clearInterval(interval);
    }, []);

    const updateLevel = async (target: string, level: LogLevel) => {
        setLoading(true);
        setFeedback(null);

        try {
            const response = await fetch("api/levels", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({ target, level })
            });

            if (!response.ok) {
                throw new Error("Failed to update level");
            }

            setTargetLevels(new Map(targetLevels.set(target, level)));
            setFeedback({
                type: "success",
                message: `Updated ${target} to ${level.toUpperCase()}`
            });

            // Clear feedback after 3 seconds
            setTimeout(() => setFeedback(null), 3000);
        } catch (err) {
            console.error("Error updating level:", err);
            setFeedback({
                type: "error",
                message: "Failed to update log level"
            });
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className={className}>
            {/* Header */}
            <div className="flex items-center gap-2 mb-4">
                <Settings className="w-4 h-4 text-muted-foreground" />
                <h3 className="font-semibold text-foreground">Level Controls</h3>
            </div>

            {/* Feedback */}
            {feedback && (
                <div className="mb-4">
                    <Badge variant={feedback.type === "success" ? "default" : "destructive"}>
                        {feedback.message}
                    </Badge>
                </div>
            )}

            {/* Target List */}
            {targets.length === 0 ? (
                <p className="text-muted-foreground text-sm">No targets discovered yet...</p>
            ) : (
                <div className="space-y-3">
                    {targets.map((target) => (
                        <div key={target} className="space-y-1.5">
                            <div className="font-mono text-xs text-muted-foreground truncate" title={target}>
                                {target}
                            </div>
                            <Select
                                value={targetLevels.get(target) || "info"}
                                onValueChange={(value) =>
                                    updateLevel(target, value as LogLevel)
                                }
                                disabled={loading}
                            >
                                <SelectTrigger className="w-full">
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    {LOG_LEVELS.map((level) => (
                                        <SelectItem
                                            key={level}
                                            value={level}
                                            className="uppercase"
                                        >
                                            {level}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}
