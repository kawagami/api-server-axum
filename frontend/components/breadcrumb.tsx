"use client";

import React from 'react';
import { Link, usePathname } from '@/i18n/navigation';

interface BreadcrumbProps {
    homeElement: React.ReactNode;
    separator: React.ReactNode;
    containerClasses: string;
    listClasses: string;
    activeClasses: string;
    capitalizeLinks: boolean;
}

const Breadcrumb = ({ homeElement, separator, containerClasses, listClasses, activeClasses, capitalizeLinks }: BreadcrumbProps) => {
    const paths = usePathname();
    const pathNames = paths.split('/').filter(path => path);

    return (
        <div>
            <ul className={containerClasses}>
                <li className={listClasses}><Link href={'/'}>{homeElement}</Link></li>
                {pathNames.length > 0 && separator}
                {pathNames.map((link, index) => {
                    const href = `/${pathNames.slice(0, index + 1).join('/')}`;
                    const itemClasses = paths === href ? `${listClasses} ${activeClasses}` : listClasses;
                    const itemLink = capitalizeLinks ? link[0].toUpperCase() + link.slice(1) : link;
                    return (
                        <React.Fragment key={index}>
                            <li className={itemClasses}>
                                <Link href={href}>{itemLink}</Link>
                            </li>
                            {pathNames.length !== index + 1 && separator}
                        </React.Fragment>
                    );
                })}
            </ul>
        </div>
    );
};

export default Breadcrumb;
