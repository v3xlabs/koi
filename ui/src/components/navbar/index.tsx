import { Link } from "@tanstack/solid-router";
import { FiLock, FiUnlock } from "solid-icons/fi";

import logo from "../../assets/kohaku.svg";
import { ConnectionButton } from "../connection";
import { NetworkWidget } from "./networks";

export const Navbar = () => (
    <div class="border-b border-background px-2 flex justify-between items-center bg-surface">
        <div class="flex items-center gap-2 py-2">
            <Link to="/" class="flex items-center gap-2">
                <div class="w-8 h-8">
                    <img src={logo} alt="Koi" class="w-full h-full object-contain" />
                </div>
                <div class="leading-none">
                    <h1 class="font-bold">Koi</h1>
                    <span class="text-muted text-sm">a privacy wallet</span>
                </div>
            </Link>
        </div>
        <div>
        </div>
        <div class="flex items-center h-full">
            <ConnectionButton />
            <NetworkWidget />
            <button class="nav-icon-button group">
                <FiUnlock class="group-hover:hidden" />
                <FiLock class="hidden group-hover:block" />
            </button>
        </div>
    </div>
);
