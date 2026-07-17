import { Link } from "@tanstack/solid-router";

import logo from "../../assets/kohaku.svg";

export const Branding = () => (
    <Link to="/" class="flex min-w-0 items-center gap-2 p-2">
        <div class="w-8 h-8 shrink-0">
            <img src={logo} alt="Koi" class="w-full h-full object-contain" />
        </div>
        <div class="leading-none min-w-0">
            <h1 class="font-bold">Koi</h1>
            <span class="text-muted text-sm whitespace-nowrap">just a wallet</span>
        </div>
    </Link>
);
