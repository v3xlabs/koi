import { Link } from "@tanstack/solid-router";
import { FiHome, FiUser } from "solid-icons/fi";
import { For } from "solid-js";

import logo from "../assets/kohaku.svg";

export const Navbar = () =>
    //

     (
        <div class="border-b px-2 py-2 flex justify-between items-center bg-surface border-b-border">
            <div class="flex items-center gap-2">
                <div class="flex items-center gap-2">
                    <Link to="/" class="w-8 h-8">
                        <img src={logo} alt="Koi" class="w-full h-full object-contain" />
                    </Link>
                    <div class="leading-none">
                        <h1 class="font-bold">Koi</h1>
                        <span class="text-muted text-sm">a privacy wallet</span>
                    </div>
                </div>
            </div>
            <div>
                Center
            </div>
            <div>
                Right
            </div>
        </div>
    );
