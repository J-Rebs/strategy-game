// Dynamic tab switching logic for the docs page with smooth fade animations
document.addEventListener('DOMContentLoaded', () => {
    const navButtons = document.querySelectorAll('.nav-btn');
    const sections = document.querySelectorAll('.doc-section');

    navButtons.forEach(button => {
        button.addEventListener('click', () => {
            const targetId = button.getAttribute('data-target');

            // 1. Remove active state from all buttons
            navButtons.forEach(btn => btn.classList.remove('active'));
            // 2. Set clicked button as active
            button.classList.add('active');

            // 3. Fade out currently active section, then show the target section
            sections.forEach(section => {
                if (section.classList.contains('active')) {
                    section.style.opacity = '0';
                    section.style.transform = 'translateY(15px)';
                    
                    setTimeout(() => {
                        section.classList.remove('active');
                        
                        // Show and fade in target section
                        const targetSection = document.getElementById(targetId);
                        targetSection.classList.add('active');
                        
                        // Force layout reflow
                        targetSection.offsetHeight;
                        
                        targetSection.style.opacity = '1';
                        targetSection.style.transform = 'translateY(0)';
                    }, 300); // matches CSS transition time
                }
            });
        });
    });
});
