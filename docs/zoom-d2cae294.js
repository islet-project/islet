document.addEventListener('DOMContentLoaded', function () {
    const images = document.querySelectorAll('.content main img[title="large-view"]');

    images.forEach(img => {
        img.style.cursor = 'zoom-in';
        img.addEventListener('click', function () {
            if (img.classList.contains('fullscreen-img')) {
                img.classList.remove('fullscreen-img');
                img.style.cursor = 'zoom-in';
            } else {
                img.classList.add('fullscreen-img');
                img.style.cursor = 'zoom-out';
            }
        });
    });
});

