import { Link } from '@tanstack/solid-router';
import logo from '../assets/kohaku.svg';
import { For } from 'solid-js';
import { FiHome, FiUser } from 'solid-icons/fi';

export const Navbar = () => {
    //

    return (
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
                <div class="flex items-center gap-2">
                    <For each={[
                        {
                            icon: FiHome,
                            label: "Home",
                            href: "/",
                        },
                        {
                            icon: FiUser,
                            label: "Account 1",
                            href: "/acc/1",
                        }
                    ]}>
                        {(item) => (
                            <Link to={item.href} class="flex items-center gap-2">
                                <item.icon class="w-3.5 h-3.5" />
                                {item.label}
                            </Link>
                        )}
                    </For>
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
};
