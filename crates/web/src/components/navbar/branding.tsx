import { Link } from "@tanstack/solid-router";

import logo from "../../assets/kohaku.svg";
import { cn } from "../input/button";

export const Branding = (props: { class?: string; tabIndex?: number; }) => (
    <Link to="/" class={cn("flex min-w-0 items-center gap-2 p-2", props.class)} tabIndex={props.tabIndex}>
        <div class="w-8 h-8 shrink-0">
            <img src={logo} alt="Koi" class="w-full h-full object-contain" />
        </div>
        <div class="leading-none min-w-0">
            <h1 class="font-bold">Koi</h1>
            <span class="text-muted text-sm whitespace-nowrap">just a wallet</span>
        </div>
    </Link>
);
