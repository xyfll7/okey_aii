import { useRef, useState } from "react";
import { Icons } from "#/components/icon";

const Copyed = ({ text, className }: { text?: string; className?: string }) => {
	const [copied, setCopied] = useState(false);
	const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

	return (
		<div 
			role="none"
			onClick={async (e: React.MouseEvent) => {
				e.stopPropagation(); 
				e.preventDefault();  
		
				if (text) {
					try {
						await navigator.clipboard.writeText(text);
						setCopied(true);

				
						if (timeoutRef.current) {
							clearTimeout(timeoutRef.current);
						}

						timeoutRef.current = setTimeout(() => {
							setCopied(false);
						}, 2000);
					} catch (err) {
						console.error("Failed to copy text: ", err);
					}
				}
			}} 
			onMouseEnter={(e: React.MouseEvent) => {
				e.stopPropagation();
			}}
			className="inline-block"
			style={{ pointerEvents: 'auto' }} 
		>
			{copied ? <Icons.tick className={className}/> : <Icons.copy className={className}/>}
		</div>
	);
};

export default Copyed;
