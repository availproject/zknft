/** @type {import('next').NextConfig} */
module.exports = {
  images: {
    formats: ['image/avif', 'image/webp'],
    remotePatterns: [
      {
        hostname: 'storage.googleapis.com',
        pathname: '/nftimagebucket/**',
        port: ''
      }
    ]
  },
  reactStrictMode: false,
};
